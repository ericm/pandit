use std::collections::LinkedList;

use dashmap::DashMap;
use redis::{Client, Commands, Connection, PubSubCommands};

use crate::services::{message::Message, Fields, ServiceResult};

pub struct Broker {
    cfg: config::Config,
    client: Client,
    conn: Connection,
    message_fields_map: DashMap<String, (Message, LinkedList<Fields>)>,
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
            message_fields_map: Default::default(),
        })
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
                .insert(name, (message, LinkedList::default()));
        }
        let name = format!("service_{}", name);
        pubsub.subscribe(name)?;
        Ok(())
    }

    pub fn receive(&mut self) -> ServiceResult<()> {
        let mut pubsub = self.conn.as_pubsub();
        loop {
            let msg = pubsub.get_message()?;
            match msg.get_channel_name() {
                "services" => {}
                name => {
                    let name = name.to_string();
                    let name = name.trim_start_matches("service_").to_string();
                    let (message, mut fields_list) = match self.message_fields_map.get(&name) {
                        Some(v) => v.to_owned(),
                        None => {
                            eprintln!("received service fields with unknown name: {}", name);
                            continue;
                        }
                    };
                    let payload = msg.get_payload_bytes();
                    let fields = message.fields_from_bytes(payload)?;
                    fields_list.push_back(fields);
                }
            }
        }
        Ok(())
    }

    fn set_default(cfg: &mut config::Config) -> ServiceResult<()> {
        cfg.set_default("redis.address", "redis://127.0.0.1/".to_string())?;
        Ok(())
    }
}
