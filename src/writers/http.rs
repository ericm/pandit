use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::services::{Fields, Handler, ServiceError, ServiceResult, Writer, WriterContext};

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
        context: WriterContext,
        fields: &Fields,
        handler: &Arc<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes> {
        let stream = self.stream.get_mut();
        let (send, _) = h2::client::handshake(stream).await?;
        let request = request_from_context(http::Version::HTTP_2, context)?;
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

fn request_from_context(
    version: http::Version,
    context: WriterContext,
) -> ServiceResult<http::request::Request<()>> {
    let mut request = http::Request::new(());
    let mut uri = http::Uri::builder().scheme("http");
    for (k, v) in context {
        match k.as_str() {
            "method" => {
                *request.method_mut() = http::Method::from_str(v.as_str())?;
                continue;
            }
            "uri" => {
                uri = uri.path_and_query(v.as_str());
                continue;
            }
            _ => {}
        }
        let name = http::header::HeaderName::from_str(k.as_str())?;
        let value = http::HeaderValue::from_str(v.as_str())?;
        request.headers_mut().insert(name, value);
    }
    let uri = uri.build()?;
    *request.uri_mut() = uri;
    *request.version_mut() = version;
    Ok(request)
}
