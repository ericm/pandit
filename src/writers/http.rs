use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::services::{Fields, Handler, ServiceError, ServiceResult, Writer};

pub struct Http2Writer {
    stream: Mutex<tokio::net::TcpStream>,
}

impl Http2Writer {
    pub fn new(stream: Mutex<tokio::net::TcpStream>) -> Http2Writer {
        Self { stream }
    }
}

#[async_trait]
impl Writer for Http2Writer {
    async fn write_request(
        &mut self,
        context: Arc<::http::request::Parts>,
        fields: &Fields,
        handler: Box<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes> {
        use ::http;
        let stream = self.stream.get_mut();
        let (send, _) = h2::client::handshake(stream).await?;
        let request = http::Request::from_parts(Arc::try_unwrap(context).unwrap(), ());
        let payload = handler.to_payload(fields).await?;

        let mut resp = send
            .ready()
            .await
            .and_then(|mut send_req| {
                let (resp, mut sender) = send_req.send_request(request, false)?;
                sender.send_data(payload, true)?;
                Ok(resp)
            })?
            .await?;

        let body = resp.body_mut();
        let body = match body.data().await {
            Some(body) => body,
            None => return Err(ServiceError::new("no body in response")),
        };
        match body {
            Ok(body) => Ok(body),
            Err(e) => Err(ServiceError::new(
                format!("error parsing body: {}", e).as_str(),
            )),
        }
    }
}
