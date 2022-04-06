use std::{str::FromStr, sync::Arc, convert::TryInto};

use async_trait::async_trait;
use hyper::{body::HttpBody, client::conn};
use tokio::sync::{Mutex, RwLock};

use crate::{
    proto::gen::format::http::HTTPVersion,
    services::{Fields, Handler, ServiceError, ServiceResult, Writer, WriterContext},
};

use super::LoadBalancer;

pub struct HttpWriter {
    client: hyper::Client<hyper::client::HttpConnector>,
    version: http::Version,
    lb: LoadBalancer,
}

impl HttpWriter {
    pub fn new(lb: LoadBalancer, version: HTTPVersion) -> HttpWriter {
        let version = match version {
            HTTPVersion::VERSION_1_0 => http::Version::HTTP_10,
            HTTPVersion::VERSION_1_1 => http::Version::HTTP_11,
            HTTPVersion::VERSION_2_0 => http::Version::HTTP_2,
        };
        let client = hyper::Client::new();
        Self {
            client,
            version,
            lb,
        }
    }
}

#[async_trait]
impl Writer for HttpWriter {
    async fn write_request(
        &mut self,
        context: WriterContext,
        fields: &Fields,
        handler: &Arc<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes> {
        let payload = handler.to_payload(fields).await?;
        let addr = self.lb.get_addr().await;
        let request = request_from_context(self.version.clone(), context, payload, addr.clone())?;
        log::info!("sending HTTP request to {}", &addr);
        let mut resp = self.client.request(request).await?;

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
    body: bytes::Bytes,
    addr: String,
) -> ServiceResult<http::request::Request<hyper::Body>> {
    let len = body.len();
    let body = hyper::Body::from(body);
    let mut builder = http::Request::builder();
    let mut uri = http::Uri::builder().scheme("http");

    for (k, v) in context {
        match k.as_str() {
            "method" => {
                builder = builder.method(http::Method::from_str(v.as_str())?);
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
        builder = builder.header(name, value)
    }

    // Set Content-Length.
    builder = builder.header(
        http::header::HeaderName::from_str("Content-Length")?,
        http::HeaderValue::from_str(len.to_string().as_str())?,
    );

    uri = uri.authority(addr);
    let uri = uri.build()?;
    builder = builder.uri(uri);
    builder = builder.version(version);
    let request = builder.body(body)?;
    Ok(request)
}
