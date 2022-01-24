use std::error::Error;
use std::sync::Arc;

use crate::services;
use bytes::Bytes;
use h2::server::{self, SendResponse};
use h2::RecvStream;
use http::Request;
use protobuf::Message;
use protofish::context::MessageRef;
use std::collections::HashMap;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tonic;

pub struct Server {
    services: Arc<services::Services>,
}

impl Server {
    pub async fn run(
        &self,
        service: services::Service,
        addr: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        loop {
            if let Ok((socket, saddr)) = listener.accept().await {
                self.serve(socket);
            }
        }
        Ok(())
    }

    async fn serve(&self, socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync + '_>> {
        let mut conn = server::handshake(socket).await?;
        while let Some(result) = conn.accept().await {
            let (req, resp) = result?;
            let services = self.services.clone();
            tokio::spawn(async move {
                if let Err(e) = Server::handle_request(services, req, resp).await {
                    println!("error handling request: {}", e);
                }
            });
        }
        Ok(())
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
        let service = path.next().unwrap();
        let method = path.next().unwrap();

        let service = services.get(service).unwrap();

        service
            .send_proto_to_local(&method.to_string(), &data[..])
            .await?;
        let response = http::Response::new(());
        let mut send = respond.send_response(response, false)?;
        Ok(())
    }
}
