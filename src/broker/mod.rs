use std::collections::HashMap;

use dashmap::DashMap;
use hyper;
use lapin::{self, types::FieldTable};

use crate::services::ServiceResult;
use config::Config;
pub struct Broker {
    conn: lapin::Connection,
    config_channel: lapin::Channel,
    cfg: config::Config,
}
impl Broker {
    pub async fn connect(mut cfg: config::Config) -> ServiceResult<Self> {
        let addr = "amqp://127.0.0.1:5672/%2f";
        let conn = lapin::Connection::connect(addr, lapin::ConnectionProperties::default()).await?;
        let config_channel = conn.create_channel().await?;
        config_channel
            .exchange_declare(
                "config",
                lapin::ExchangeKind::Fanout,
                lapin::options::ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        cfg.set_default("broker.rest_api", "http://localhost:15672".to_string())?;
        Ok(Self {
            conn,
            config_channel,
            cfg,
        })
    }

    pub async fn get_nodes(&self) -> ServiceResult<()> {
        let client = hyper::Client::new();
        let uri = self.cfg.get_str("rest_api")?;
        use std::convert::TryFrom;
        let resp: hyper::Response<Vec<u8>> = client.get(hyper::Uri::try_from(uri)?).await?;
        let body = serde_json::from_slice(resp.body());
        Ok(())
    }
}
