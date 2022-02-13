use std::collections::HashMap;

use dashmap::DashMap;
use http_auth_basic::Credentials;
use hyper::{self, body::HttpBody};
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
        Self::set_default(&mut cfg)?;
        let conn = lapin::Connection::connect(
            cfg.get_str("broker.address")?.as_str(),
            lapin::ConnectionProperties::default(),
        )
        .await?;
        let config_channel = conn.create_channel().await?;
        config_channel
            .exchange_declare(
                "config",
                lapin::ExchangeKind::Fanout,
                lapin::options::ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            conn,
            config_channel,
            cfg,
        })
    }

    fn set_default(cfg: &mut config::Config) -> ServiceResult<()> {
        cfg.set_default("broker.rest_api", "http://localhost:15672".to_string())?;
        cfg.set_default("broker.address", "amqp://127.0.0.1:5672/%2f".to_string())?;
        cfg.set_default("broker.username", "guest".to_string())?;
        cfg.set_default("broker.password", "guest".to_string())?;
        Ok(())
    }

    async fn get(&self, endpoint: &str) -> ServiceResult<serde_json::Value> {
        let client = hyper::Client::new();
        let mut uri = self.cfg.get_str("broker.rest_api")?;
        uri.push_str(endpoint);

        let creds = Credentials::new(
            self.cfg.get_str("broker.username")?.as_str(),
            self.cfg.get_str("broker.password")?.as_str(),
        )
        .as_http_header();
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(uri.as_str())
            .header(hyper::header::AUTHORIZATION, creds)
            .body(hyper::Body::default())?;
        let mut resp = client.request(req).await?;
        let mut data = Vec::<u8>::new();
        while let Some(chunk) = resp.body_mut().data().await {
            data.extend(chunk?);
        }

        Ok(serde_json::from_slice::<serde_json::Value>(&data[..])?)
    }

    pub async fn get_nodes(&self) -> ServiceResult<serde_json::Value> {
        self.get("/api/nodes").await
    }
}

mod tests {
    #[tokio::test]
    async fn get_nodes() {
        let broker = super::Broker::connect(config::Config::default())
            .await
            .unwrap();
        println!("{:?}", broker.get_nodes().await.unwrap());
    }
}
