#![feature(destructuring_assignment)]
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::sync::Arc;

use crate::broker::{Broker, RemoteSender};
use crate::services::{self, Sender, Service, ServiceError};
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use futures::Future;
use h2::server::{self, SendResponse};
use h2::RecvStream;
use http::{HeaderMap, HeaderValue, Request};
use protobuf::Message;
use std::collections::HashMap;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::ctrl_c;
use tokio::sync::{RwLock, RwLockReadGuard};
use tonic;

#[async_trait]
pub trait Server {
    async fn run(&self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("gRPC server starting on: {}...", addr);
        let listener = TcpListener::bind(addr.clone()).await?;
        log::info!("gRPC server listening on: {}", addr);
        loop {
            tokio::select! {
                _ = ctrl_c() => {
                    return Ok(());
                },
               Ok((socket, saddr)) = listener.accept() => {
                match self.serve(socket).await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!(
                            "error parsing packet from {}: {}",
                            saddr.to_string(),
                            e.to_string()
                        );
                    }
                }
               }
            }
        }
    }
    async fn serve(&self, socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync + '_>>;
}

pub struct IntraServer {
    services: Arc<RwLock<services::Services>>,
    broker: Arc<Broker>,
}

#[async_trait]
impl Server for IntraServer {
    async fn serve(&self, socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync + '_>> {
        let mut conn = server::handshake(socket).await?;
        while let Some(result) = conn.accept().await {
            let (req, mut send_resp) = result?;
            let services = self.services.clone();
            let broker = self.broker.clone();
            tokio::spawn(async move {
                let response = http::Response::new(());
                let mut trailers = HeaderMap::new();
                let services = services.read().await;
                let resp_raw = IntraServer::handle_request(services, req, broker).await;

                let mut send = send_resp.send_response(response, false).unwrap();
                match resp_raw {
                    Ok(payload) => {
                        let mut payload = payload.to_vec();
                        let payload_len: u32 = (payload.len() - 5).try_into().unwrap();

                        // Write payload length to gRPC header.
                        let grpc_header = payload_len.to_be_bytes();
                        let mut grpc_header = grpc_header.iter();
                        let header = payload[1..5].as_mut();
                        for byte in header.iter_mut() {
                            *byte = *grpc_header.next().unwrap();
                        }

                        trailers.insert("grpc-status", HeaderValue::from(0i16));
                        send.send_data(Bytes::copy_from_slice(&payload[..]), false)
                            .unwrap();
                    }
                    Err(err) => {
                        log::error!("error occured handling request: {:?}", err);
                        trailers.insert("grpc-status", HeaderValue::from(13i16));
                        trailers.insert(
                            "grpc-message",
                            HeaderValue::try_from(format!(
                                "an internal error occurred in pandit: {:?}",
                                err
                            ))
                            .unwrap(),
                        );
                    }
                }
                send.send_trailers(trailers).unwrap();
            });
        }
        Ok(())
    }
}

impl IntraServer {
    pub fn new(broker: Arc<Broker>) -> Self {
        Self {
            services: Arc::new(RwLock::new(services::Services::default())),
            broker,
        }
    }

    pub async fn add_servivce(&self, name: String, service: services::Service) {
        log::info!("adding service to intra-server: {}", &name);
        let services = self.services.write().await;
        services.insert(name.clone(), service);
    }

    pub async fn remove_service(&self, name: &String) {
        log::info!("removing service from intra-server: {}", &name);
        let services = self.services.write().await;
        services.remove(name);
    }

    async fn handle_request(
        services: RwLockReadGuard<'_, DashMap<String, Service>>,
        mut request: Request<RecvStream>,
        broker: Arc<Broker>,
    ) -> Result<Bytes, Box<dyn Error>> {
        let body = request.body_mut();
        let data: Vec<u8> = body
            .data()
            .await
            .iter()
            .map(|data| {
                let data = data.as_ref().unwrap();
                let _ = body.flow_control().release_capacity(data.len());
                data.to_vec()
            })
            .flatten()
            .collect();
        let path = request.uri().to_string();
        let mut path = path.rsplit("/");
        let method = path.next().unwrap();
        let service = {
            let fqdn = path.next().unwrap();
            fqdn.rsplit(".").next().unwrap()
        };
        let service_name = service.to_string();
        let method_name = method.to_string();
        log::info!("request for {}_{}", &service_name, &method_name);

        // subscribe for future cache.
        if !broker.is_subbed(&service_name, &method_name) {
            log::info!(
                "subscribing to cache for {}_{}",
                &service_name,
                &method_name
            );
            broker.sub_service(&service_name, &method_name).await?;
        }
        let mut _remote_sender: RemoteSender;
        let mut _service: RefMut<String, Service>;
        let service: &mut dyn Sender = match services.get_mut(service) {
            Some(s) => {
                log::info!("found service {} on this node", &service_name);
                _service = s;
                _service.value_mut()
            }
            None => {
                log::info!(
                    "found service {} on other node, will delegate if no cache hit",
                    &service_name
                );
                // send to other node.
                _remote_sender = broker.get_remote_sender(&service_name)?;
                &mut _remote_sender
            }
        };

        // Probe cache for in date cached data.
        if let Some(cached_data) = service
            .probe_cache(&service_name, &method_name, &data[..])
            .await?
        {
            log::info!("found cache hit for {}_{}", &service_name, &method_name);
            return Ok(cached_data);
        }

        let resp_raw = service.send(&service_name, &method_name, &data[..]).await;
        match resp_raw {
            Ok(v) => Ok(v),
            Err(err) => Err(ServiceError::new(
                format!("an error occured sending the payload: {:?}", err).as_str(),
            )),
        }
    }
}
