#![feature(destructuring_assignment)]
use std::error::Error;
use std::sync::Arc;

use crate::services::{self, ServiceError};
use async_trait::async_trait;
use bytes::Bytes;
use h2::server::{self, SendResponse};
use h2::RecvStream;
use http::Request;
use protobuf::Message;
use std::collections::HashMap;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tonic;

#[async_trait]
pub trait Server {
    async fn run(&self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        loop {
            if let Ok((socket, saddr)) = listener.accept().await {
                match self.serve(socket).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!(
                            "error parsing packet from {}: {}",
                            saddr.to_string(),
                            e.to_string()
                        );
                    }
                }
            }
        }
        Ok(())
    }
    async fn serve(&self, socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync + '_>>;
}

pub struct IntraServer {
    services: Arc<services::Services>,
}

impl Default for IntraServer {
    fn default() -> Self {
        Self {
            services: Arc::new(services::Services::default()),
        }
    }
}

#[async_trait]
impl Server for IntraServer {
    async fn serve(&self, socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync + '_>> {
        let mut conn = server::handshake(socket).await?;
        while let Some(result) = conn.accept().await {
            let (req, resp) = result?;
            let services = self.services.clone();
            tokio::spawn(async move {
                if let Err(e) = IntraServer::handle_request(services, req, resp).await {
                    println!("error handling request: {}", e);
                }
            });
        }
        Ok(())
    }
}

impl IntraServer {
    pub fn add_servivce(&mut self, name: String, service: services::Service) {
        self.services.insert(name.clone(), service);
    }

    async fn handle_request(
        services: Arc<services::Services>,
        mut request: Request<RecvStream>,
        mut respond: SendResponse<Bytes>,
    ) -> Result<(), Box<dyn Error>> {
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

        let mut service = match services.get_mut(service) {
            Some(s) => s,
            None => {
                return Err(ServiceError::new(
                    format!("no service known as: {}", service).as_str(),
                ));
            }
        };

        let resp_payload = service
            .send_proto_to_local(&service_name, &method.to_string(), &data[..])
            .await?;
        let response = http::Response::new(());
        let mut send = respond.send_response(response, false)?;
        send.send_data(resp_payload, true)?;
        Ok(())
    }
}
