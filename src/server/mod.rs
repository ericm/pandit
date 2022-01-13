use std::error::Error;
use std::sync::Arc;

use crate::services;
use bytes::Bytes;
use h2::server::{self, SendResponse};
use h2::RecvStream;
use http::Request;
use protobuf::Message;
use tokio;
use tokio::net::{TcpListener, TcpStream};
use tonic;
pub struct Server {}

pub async fn run(
    service: services::Service,
    addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        if let Ok((socket, saddr)) = listener.accept().await {
            tokio::spawn(async move {
                serve(socket);
            });
        }
    }
    Ok(())
}

async fn serve(socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut conn = server::handshake(socket).await?;
    while let Some(result) = conn.accept().await {
        let (req, resp) = result?;
        tokio::spawn(async move {
            if let Err(e) = handle_request(req, resp).await {
                println!("error handling request: {}", e);
            }
        });
    }
    Ok(())
}

struct Codec {}

impl tonic::codec::ProstCodec for Codec {}

async fn handle_request(
    mut request: Request<RecvStream>,
    mut respond: SendResponse<Bytes>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
    let mut request = protobuf::CodedInputStream::from_bytes(&data[..]);
    let codec = tonic::codec::ProstCodec::default();
    let (field_num, wire_type) = request.read_tag_unpack()?;
    let response = http::Response::new(());
    let mut send = respond.send_response(response, false)?;
    Ok(())
}
