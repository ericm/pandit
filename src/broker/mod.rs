use std::collections::LinkedList;
use std::env::current_dir;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use access_json::JSONQuery;
use async_trait::async_trait;
use dashmap::mapref::one::Ref;
use dashmap::{DashMap, DashSet};
use futures::StreamExt;
use grpcio::{ChannelBuilder, EnvBuilder};
use hyper::body::HttpBody;
use protobuf::well_known_types::Field;
use redis::cluster::ClusterClient;
use redis::{Client, Commands, Connection, Msg, PubSubCommands};
use tokio::time::sleep;

use crate::api::{add_service_from_file, K8sHandler};
use crate::services::base::CacheOptions;
use crate::services::value::Value;
use crate::services::{message::Message, Fields, ServiceResult};
use crate::services::{FieldsMap, Method, Sender, Service, ServiceError};

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

pub struct RemoteSender {
    addr: String,
}

#[async_trait]
impl Sender for RemoteSender {
    async fn send(
        &mut self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<bytes::Bytes> {
        let client = hyper::client::Client::new();
        let req = hyper::Request::builder()
            .uri(format!("http://{}/{}/{}", self.addr, service_name, method))
            .method("POST")
            .body(hyper::Body::from(data.to_vec()))?;
        let mut resp = client.request(req).await?;
        let body = resp.body_mut();
        let body = match body.data().await {
            Some(body) => body,
            None => return Err(ServiceError::new("no body in response")),
        };
        Ok(body?)
    }
}

pub struct Broker {
    client: Client,
    method_fields_map: Arc<DashMap<String, CachedMessage>>,
    subbed: Arc<DashSet<String>>,
    host_addr: String,
}

impl Broker {
    pub fn connect(mut cfg: config::Config, host_addr: String) -> ServiceResult<Self> {
        Self::set_default(&mut cfg)?;
        let client = redis::Client::open(cfg.get_str("redis.address")?.as_str())?;
        let mut conn = client.get_connection()?;
        {
            let mut pubsub = conn.as_pubsub();
            pubsub.subscribe("services")?;
        }
        conn.rpush("hosts", &host_addr)?;
        Ok(Self {
            host_addr,
            client,
            method_fields_map: Arc::new(Default::default()),
            subbed: Arc::new(DashSet::new()),
        })
    }

    pub fn get_hosts(&self) -> ServiceResult<Vec<String>> {
        let mut conn = self.client.get_connection()?;
        let hosts: Vec<String> = conn.get("hosts")?;
        Ok(hosts)
    }

    pub fn is_subbed(&self, name: &String) -> bool {
        self.subbed.contains(name)
    }

    pub fn probe_cache(
        &self,
        service_name: &String,
        method_name: &String,
        primary_key: &Value,
    ) -> ServiceResult<Option<Fields>> {
        let name = format!("{}_{}", service_name, method_name);
        let val = self.get_entry(&name)?;
        let message = val.value();

        // Look for existing entry or return "no hit".
        let entry = match message.fields_for_key.get(primary_key) {
            Some(v) => v,
            None => return Ok(None),
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
        Ok(Some(entry.fields.clone()))
    }

    pub fn publish_service(&self, name: &String, service: &Service) -> ServiceResult<()> {
        let mut conn = self.client.get_connection()?;
        {
            let key = format!("default_{}", name);
            let value = serde_json::to_vec(&service.default_cache)?;
            conn.set(key, value)?;
        }
        {
            let key = format!("host_{}", name);
            let value = &self.host_addr;
            conn.set(key, value)?;
        }
        for method in service.methods.iter() {
            {
                let key = format!("config_{}_{}", name, method.key());
                let value = serde_json::to_vec(method.value())?;
                conn.set(key, value)?;
            }
        }
        for message in service.messages.iter() {
            let key = format!("message_{}_{}", name, message.key());
            let value = serde_json::to_vec(message.value())?;
            conn.set(key, value)?;
        }
        Ok(())
    }

    pub async fn remove_service(&self, name: &String, pod_name: &String) -> ServiceResult<()> {
        use redis::AsyncCommands;
        let mut conn = self.client.get_async_connection().await?;
        {
            let key = format!("default_{}", name);
            conn.del(key).await?;
        }
        {
            let key = format!("host_{}", name);
            conn.del(key).await?;
        }

        conn.publish("evicted", (name, pod_name)).await?;
        Ok(())
    }

    pub fn get_remote_sender(&self, service_name: &String) -> ServiceResult<RemoteSender> {
        let mut conn = self.client.get_connection()?;
        let addr: String = conn.get(format!("host_{}", service_name))?;
        Ok(RemoteSender { addr })
    }

    pub fn sub_service(&self, service_name: &String, method_name: &String) -> ServiceResult<()> {
        let mut conn = self.client.get_connection()?;

        let method: Method = {
            let rv: Vec<u8> = conn.get(format!("config_{}_{}", service_name, method_name))?;
            serde_json::from_slice(&rv[..])?
        };

        let message = {
            let parents = {
                let keys: Vec<String> = conn.keys(format!("message_{}_*", service_name))?;
                let out = Arc::new(DashMap::<String, Message>::new());
                for key in keys {
                    let rv: Vec<u8> = conn.get(format!("message_{}_{}", service_name, key))?;
                    let mut message: Message = serde_json::from_slice(&rv[..])?;
                    message.parent = out.clone();
                    out.insert(key, message);
                }
                out
            };
            let val = parents.get(&method.output_message);
            let val = val.ok_or("no output message")?;
            val.value().clone()
        };

        let default_cache: CacheOptions = {
            let rv: Vec<u8> = conn.get(format!("default_{}", service_name))?;
            serde_json::from_slice(&rv[..])?
        };
        let name = format!("{}_{}", service_name, method_name);
        let sub_name = format!("service_{}", name.clone());
        self.method_fields_map.insert(
            name,
            CachedMessage {
                message: message.to_owned(),
                cache: method.cache.clone().or(Some(default_cache.clone())),
                primary_key: method
                    .primary_key
                    .to_owned()
                    .ok_or(ServiceError::new(format!("no primary key").as_str()))?,
                fields_for_key: Arc::new(DashMap::new()),
            },
        );
        self.subbed.insert(sub_name.clone());
        {
            let mut pubsub = conn.as_pubsub();
            pubsub.subscribe(sub_name)?;
        }
        Ok(())
    }

    pub async fn publish_cache(
        &self,
        service_name: &String,
        method_name: &String,
        fields: Fields,
    ) -> ServiceResult<()> {
        let name = format!("{}_{}", service_name, method_name);
        let service_name = format!("service_{}", name.clone());
        let buf: Vec<u8> = Vec::with_capacity(1000);
        use bytes::BufMut;
        let mut buf = buf.writer();
        let primary_key: Value;
        {
            let cached = self.get_entry(&name)?;
            let prim_key_opt = fields.map.get(&cached.primary_key);
            primary_key = match prim_key_opt {
                Some(v) => v.to_owned().ok_or(ServiceError::new(
                    format!("no value for primary key entry: {}", cached.primary_key).as_str(),
                ))?,
                None => return Ok(()),
            };
            let fields = self.filter_fields(&fields, &cached.value().message)?;
            {
                let mut output = protobuf::CodedOutputStream::new(&mut buf);
                cached
                    .message
                    .write_bytes_from_fields(&mut output, &fields)?;
            }
        }
        let buf = buf.into_inner();
        let to_publish = serde_json::to_vec(&(primary_key, buf))?;

        let mut pubsub = self.client.get_async_connection().await?;
        use redis::AsyncCommands;
        pubsub.publish(service_name, to_publish).await?;
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
                None => return Ok(fields.to_owned()),
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

    pub async fn receive(&self, k8s_handler: Option<Arc<K8sHandler>>) -> ServiceResult<()> {
        let msg = {
            let pubsub = self.client.get_async_connection().await?;
            let mut pubsub = pubsub.into_pubsub();
            let msg: Msg;
            loop {
                for service in self.subbed.iter() {
                    let service = service.clone();
                    pubsub.subscribe(service).await?;
                }
                let mut on_msg = pubsub.on_message();
                msg = tokio::select! {
                    v = on_msg.next() => match v {
                        Some(v) => v,
                        None => return Ok(()),
                    },
                    _ = sleep(Duration::from_secs(30)) => {
                        continue;
                    },
                };
                break;
            }
            msg
        };
        match msg.get_channel_name() {
            "evicted" => match k8s_handler {
                Some(handler) => {
                    self.handle_eviction(handler, msg.get_payload()?);
                }
                None => {
                    log::error!("pod eviction received but no k8s handler")
                }
            },
            name => match self.parse_service_fields(name, &msg) {
                Ok(_) => {}
                Err(err) => {
                    log::error!(
                        "error occured parsing service with name {}: {:?}",
                        name,
                        err
                    );
                }
            },
        }
        Ok(())
    }

    fn handle_eviction(&self, handler: Arc<K8sHandler>, (service_name, pod): (String, String)) {
        let host_addr = self.host_addr.clone();
        tokio::spawn(async move {
            for _ in 0..10 {
                let on_current = handler.is_pod_on_current(&pod).await.unwrap();
                if on_current {
                    log::info!("pod '{}' is now on this node", pod);
                    // Add service on this node
                    {
                        let mut path = current_dir().unwrap().join(service_name);
                        path.set_extension("pandit_service");
                        let mut _pods = Default::default();
                        let client = {
                            let env = Arc::new(EnvBuilder::new().build());
                            let ch = ChannelBuilder::new(env).connect(host_addr.as_str());
                            api_proto::api_grpc::ApiClient::new(ch)
                        };
                        add_service_from_file(path, &Some(handler), &mut _pods, &client)
                            .await
                            .unwrap();
                    }
                    return;
                }
                sleep(Duration::from_secs(6)).await;
            }
            log::debug!("pod '{}' was not found on this node", pod);
        });
    }

    fn parse_service_fields(&self, name: &str, msg: &redis::Msg) -> ServiceResult<()> {
        let name = name.to_string();
        let name = name.trim_start_matches("service_").to_string();
        match self.method_fields_map.get_mut(&name) {
            Some(mut v) => {
                let cached = v.value_mut();
                let (primary_key, payload): (Value, Vec<u8>) =
                    serde_json::from_slice(msg.get_payload_bytes())?;
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
