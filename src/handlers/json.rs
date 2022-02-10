use std::iter::FromIterator;

use crate::services::http;
use access_json::JSONQuery;
use async_trait::async_trait;

use crate::services::{Fields, Handler, ServiceError, ServiceResult};

pub struct JsonHandler {
    pub method: http::api::Pattern,
    prog: JSONQuery,
}

impl JsonHandler {
    pub fn new(method: http::api::Pattern, path: String) -> Self {
        Self {
            method,
            prog: JSONQuery::parse(path.as_str()).unwrap(),
        }
    }
}

#[async_trait]
impl Handler for JsonHandler {
    fn from_payload(&self, buf: bytes::Bytes) -> ServiceResult<Fields> {
        use bytes::Buf;
        let json: serde_json::Value = serde_json::from_reader(buf.reader())?;
        let pr = self.prog.execute(&json)?;
        let result = pr.ok_or(ServiceError::new("no result"))?;
        println!("json");
        Ok(serde_json::value::from_value(result)?)
    }

    async fn to_payload(&self, fields: &Fields) -> ServiceResult<bytes::Bytes> {
        match serde_json::to_vec(fields) {
            Ok(data) => Ok(bytes::Bytes::from_iter(data)),

            Err(e) => Err(ServiceError::new(
                format!("to_payload json failed: {}", e.to_string()).as_str(),
            )),
        }
    }
}
