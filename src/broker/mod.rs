use std::collections::LinkedList;
use std::sync::Arc;
use std::time::SystemTime;

use access_json::JSONQuery;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use protobuf::well_known_types::Field;
use redis::cluster::ClusterClient;
use redis::{Client, Commands, Connection, PubSubCommands};

use crate::services::base::CacheOptions;
use crate::services::value::Value;
use crate::services::{message::Message, Fields, ServiceResult};
use crate::services::{FieldsMap, Method, ServiceError};

struct CachedFields {
    fields: Fields,
    timestamp: SystemTime,
}

struct CachedMessage {
    message: Message,
    cache: Option<CacheOptions>,
    fields_for_key: Arc<DashMap<Value, CachedFields>>,
    primary_key: String,
}

impl CachedMessage {
    // pub fn set_fields_from_bytes(&mut self, buf: &[u8]) -> ServiceResult<()> {
    //     let fields = self.message.fields_from_bytes(buf)?;
    //     for (key, value) in fields.map {
    //         let proto = self.message.fields_by_name.get(&key);
    //         let proto = proto.ok_or(ServiceError::new(
    //             format!("no field named: {}", key).as_str(),
    //         ))?;
    //         let proto = proto.value();
    //         let cache = self
    //             .cache
    //             .as_ref()
    //             .or(proto.cache.as_ref())
    //             .unwrap_or_default();
    //         if !cache. {}
    //     }
    //     Ok(())
    // }
}

pub struct Broker {
    cfg: config::Config,
    client: Client,
    conn: Connection,
    method_fields_map: Arc<DashMap<String, CachedMessage>>,
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
            method_fields_map: Arc::new(Default::default()),
        })
    }

    pub fn probe_cache(
        &self,
        service_name: &String,
        method_name: &String,
        primary_key: &Value,
        path: String,
    ) -> ServiceResult<Option<Fields>> {
        let name = format!("{}_{}", service_name, method_name);
        let val = self.get_entry(&name)?;
        let message = val.value();

        let entry = {
            let val = message.fields_for_key.get(primary_key);
            val.ok_or(ServiceError::new(
                format!("no primary key entry for: {:?}", primary_key).as_str(),
            ))?
        };

        let now = SystemTime::now();
        let cached = match &message.cache {
            Some(v) => v,
            None => return Ok(None),
        };
        let diff = now.duration_since(entry.timestamp)?;
        if diff.as_secs() > cached.cache_time {
            return Ok(None);
        }
        let parse = JSONQuery::parse(path.as_str())?;
        let res = parse.execute(&entry.fields)?;
        match res {
            Some(val) => {
                let fields: Fields = serde_json::value::from_value(val)?;
                Ok(Some(fields))
            }
            None => Ok(None),
        }
    }

    pub fn sub_service(
        &mut self,
        name: &String,
        service: &crate::services::Service,
    ) -> ServiceResult<()> {
        let mut pubsub = self.conn.as_pubsub();
        for method in service.methods.iter() {
            let message = service
                .messages
                .get(&method.value().output_message)
                .unwrap();
            let name = format!("{}_{}", name.clone(), method.key());
            self.method_fields_map.insert(
                name,
                CachedMessage {
                    message: message.to_owned(),
                    cache: method
                        .value()
                        .cache
                        .clone()
                        .or(Some(service.default_cache.clone())),
                    primary_key: method
                        .primary_key
                        .to_owned()
                        .ok_or(ServiceError::new(format!("no primary key").as_str()))?,
                    fields_for_key: Arc::new(DashMap::new()),
                },
            );
        }
        let name = format!("service_{}", name.clone());
        pubsub.subscribe(name)?;
        Ok(())
    }

    pub fn publish_cache(
        &mut self,
        service_name: &String,
        method_name: &String,
        fields: Fields,
    ) -> ServiceResult<()> {
        let name = format!("{}_{}", service_name, method_name);
        let buf: Vec<u8> = Vec::with_capacity(1000);
        use bytes::BufMut;
        let mut buf = buf.writer();
        let primary_key: Value;
        {
            let cached = self.get_entry(&name)?;
            primary_key = fields
                .map
                .get(&cached.primary_key)
                .ok_or(ServiceError::new(
                    format!("no primary key entry: {}", cached.primary_key).as_str(),
                ))?
                .to_owned()
                .ok_or(ServiceError::new(
                    format!("no value for primary key entry: {}", cached.primary_key).as_str(),
                ))?;
            let fields = self.filter_fields(&fields, &cached.value().message)?;
            {
                let mut output = protobuf::CodedOutputStream::new(&mut buf);
                cached
                    .message
                    .write_bytes_from_fields(&mut output, &fields)?;
            }
        }
        let buf = buf.into_inner();
        let to_publish = (primary_key, buf);
        self.conn.publish(name, to_publish)?;
        Ok(())
    }

    fn filter_fields(&self, fields: &Fields, cached: &Message) -> ServiceResult<Fields> {
        let map: FieldsMap = FieldsMap::new();
        let proto = &cached.fields_by_name;
        for entry in &fields.map {
            let value = match entry.value() {
                Some(v) => v,
                None => continue,
            };
            let field_proto = proto.get(entry.key()).ok_or(ServiceError::new(
                format!("no value found for: {}", entry.key()).as_str(),
            ))?;
            match &field_proto.cache {
                Some(opts) => {
                    if opts.disable {
                        continue;
                    }
                }
                None => todo!(),
            }
            match value {
                Value::Message(fields) => {
                    let message_type = field_proto.descriptor.get_type_name().to_string();
                    let cached = cached.parent.get(&message_type).ok_or(ServiceError::new(
                        format!("no message found for type: {}", entry.key()).as_str(),
                    ))?;
                    map.insert(
                        entry.key().clone(),
                        Some(Value::Message(self.filter_fields(fields, cached.value())?)),
                    );
                }
                v => {
                    map.insert(entry.key().clone(), Some(v.clone()));
                }
            };
        }
        Ok(Fields::new(map))
    }

    fn get_entry(&self, name: &String) -> ServiceResult<Ref<String, CachedMessage>> {
        self.method_fields_map.get(name).ok_or(ServiceError::new(
            format!("no value found for: {}", name).as_str(),
        ))
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
        match self.method_fields_map.get_mut(&name) {
            Some(mut v) => {
                let cached = v.value_mut();
                let (primary_key, payload) = msg.get_payload::<(Value, Vec<u8>)>()?;
                let fields_map = cached.fields_for_key.clone();
                let cached_fields = CachedFields {
                    fields: cached.message.fields_from_bytes(&payload[..])?,
                    timestamp: SystemTime::now(),
                };
                fields_map.insert(primary_key, cached_fields);
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
