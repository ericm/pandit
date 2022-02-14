use std::collections::LinkedList;
use std::sync::Arc;

use access_json::JSONQuery;
use dashmap::DashMap;
use protobuf::well_known_types::Field;
use redis::cluster::ClusterClient;
use redis::{Client, Commands, Connection, PubSubCommands};

use crate::services::ServiceError;
use crate::services::{message::Message, Fields, ServiceResult};

pub struct Broker {
    cfg: config::Config,
    client: Client,
    conn: Connection,
    message_fields_map: Arc<DashMap<String, (Message, Fields)>>,
}

impl Broker {
    pub fn connect(mut cfg: config::Config) -> ServiceResult<Self> {
        Self::set_default(&mut cfg)?;
        let client = redis::Client::open(cfg.get_str("redis.address")?.as_str())?;
        let mut conn = client.get_connection()?;
        {
            let mut pubsub = conn.as_pubsub();
            pubsub.subscribe("services")?;
        }
        Ok(Self {
            cfg,
            client,
            conn,
            message_fields_map: Arc::new(Default::default()),
        })
    }

    pub fn probe_cache(
        &self,
        service_name: String,
        message_name: String,
        path: String,
    ) -> ServiceResult<Option<Fields>> {
        let name = format!("{}_{}", service_name, message_name);
        let val = self.message_fields_map.get(&name).ok_or(ServiceError::new(
            format!("no value found for: {}", name).as_str(),
        ))?;
        let (_, fields) = val.value();
        let parse = JSONQuery::parse(path.as_str())?;
        let res = parse.execute(fields)?;
        match res {
            Some(val) => {
                let fields: Fields = serde_json::value::from_value(val)?;
                Ok(Some(fields))
            }
            None => Ok(None),
        }
    }

    pub fn sub_client(
        &mut self,
        name: &str,
        messages: DashMap<String, Message>,
    ) -> ServiceResult<()> {
        let mut pubsub = self.conn.as_pubsub();
        for (message_name, message) in messages {
            let name = format!("{}_{}", name, message_name);
            self.message_fields_map
                .insert(name, (message, Fields::new(Default::default())));
        }
        let name = format!("service_{}", name);
        pubsub.subscribe(name)?;
        Ok(())
    }

    pub fn receive(&mut self) -> ServiceResult<()> {
        loop {
            let msg = {
                let mut pubsub = self.conn.as_pubsub();
                pubsub.get_message()
            }?;
            match msg.get_channel_name() {
                "services" => {}
                name => match self.parse_service_fields(name, &msg) {
                    Ok(_) => continue,
                    Err(err) => {
                        eprintln!(
                            "error occured parsing service with name {}: {:?}",
                            name, err
                        );
                    }
                },
            }
        }
        Ok(())
    }

    fn parse_service_fields(&mut self, name: &str, msg: &redis::Msg) -> ServiceResult<()> {
        let name = name.to_string();
        let name = name.trim_start_matches("service_").to_string();
        match self.message_fields_map.get_mut(&name) {
            Some(mut v) => {
                let (message, fields) = v.value_mut();
                let payload = msg.get_payload_bytes();
                *fields = message.fields_from_bytes(payload)?;
                Ok(())
            }
            None => Err(ServiceError::new(
                format!("received service fields with unknown name: {}", name).as_str(),
            )),
        }
    }

    fn set_default(cfg: &mut config::Config) -> ServiceResult<()> {
        cfg.set_default("redis.address", "redis://127.0.0.1/".to_string())?;
        cfg.set_default("cluster.addresses", vec!["redis://127.0.0.1/"])?;
        Ok(())
    }
}
