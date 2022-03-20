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
use k8s_openapi::api::core::v1::Pod;
use kube::runtime::utils::try_flatten_applied;
use kube::runtime::watcher;
use protobuf::well_known_types::Field;
use redis::cluster::ClusterClient;
use redis::{Client, Commands, Connection, Msg, PubSubCommands};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use crate::api::{add_service_from_file, K8sHandler};
use crate::server::IntraServer;
use crate::services::base::CacheOptions;
use crate::services::value::Value;
use crate::services::{message::Message, Fields, ServiceResult};
use crate::services::{FieldsMap, Method, Sender, Service, ServiceError};

struct CachedFields {
    fields: Fields,
    data: bytes::Bytes,
    timestamp: SystemTime,
}

#[derive(Clone)]
struct CachedMessage {
    message: Message,
    cache: Option<CacheOptions>,
    fields_for_key: Arc<DashMap<Value, CachedFields>>,
    primary_key: String,
}

impl CachedMessage {
    fn check_cache(&self, primary_key: &Value) -> ServiceResult<Option<bytes::Bytes>> {
        // Look for existing entry or return "no hit".
        log::info!(
            "checking local cache in {} for {} = {:?}",
            self.message.path,
            self.primary_key,
            primary_key
        );
        let entry = match self.fields_for_key.get(primary_key) {
            Some(v) => v,
            None => {
                log::info!(
                    "no cache found for {:?}. Cache available: {}",
                    primary_key,
                    self.fields_for_key.len(),
                );
                return Ok(None);
            }
        };

        let now = SystemTime::now();
        let cached = match &self.cache {
            Some(v) => v,
            None => {
                log::info!("no cache config for {:?}", self.message.path);
                return Ok(None);
            }
        };
        let diff = now.duration_since(entry.timestamp)?;
        if diff.as_secs() > cached.cache_time {
            log::info!("cache found for {:?} but it was out of date", primary_key);
            return Ok(None);
        }
        Ok(Some(entry.data.clone()))
    }
}

pub struct RemoteSender {
    addr: String,
    method_fields_map: Arc<DashMap<String, CachedMessage>>,
}

#[async_trait]
impl Sender for RemoteSender {
    async fn send(
        &mut self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<bytes::Bytes> {
        use h2::*;
        log::info!(
            "Delegating request for {}_{} to {}",
            service_name,
            method,
            self.addr
        );
        let tcp = TcpStream::connect(&self.addr).await?;
        let (mut client, connection) = client::handshake(tcp).await?;
        tokio::spawn(async move {
            connection.await.unwrap();
        });
        let req = http::Request::builder()
            .uri(format!("http://{}/{}/{}", self.addr, service_name, method))
            .version(http::Version::HTTP_2)
            .method("POST")
            .body(())?;
        let (response, mut send) = client.send_request(req, false)?;
        send.send_data(bytes::Bytes::copy_from_slice(data), true)?;
        let (_, mut body) = response.await?.into_parts();
        let body = match body.data().await {
            Some(body) => body,
            None => return Err(ServiceError::new("no body in response")),
        };
        Ok(body?)
    }

    async fn probe_cache(
        &self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<Option<bytes::Bytes>> {
        let name = format!("{}_{}", service_name, method);
        let message = self
            .method_fields_map
            .get(&name)
            .ok_or(format!("no entry for service method: {}", &name))?;
        let fields = message.message.fields_from_bytes(data)?;

        let primary_key = fields.map.get(&message.primary_key).ok_or(format!(
            "no field for primary key: {}",
            &message.primary_key
        ))?;
        let primary_key = primary_key.as_ref().ok_or("no value for primary key")?;

        message.check_cache(primary_key)
    }
}

pub struct Broker {
    client: Client,
    method_fields_map: Arc<DashMap<String, CachedMessage>>,
    subbed: Arc<DashSet<String>>,
    host_addr: String,
    subbed_tx: mpsc::Sender<()>,
    subbed_rx: Arc<RwLock<mpsc::Receiver<()>>>,
    pods: Arc<DashMap<String, String>>,
}

impl Broker {
    pub fn connect(mut cfg: config::Config, host_addr: String) -> ServiceResult<Self> {
        Self::set_default(&mut cfg)?;
        let (subbed_tx, subbed_rx) = mpsc::channel(10000);
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
            subbed_rx: Arc::new(RwLock::new(subbed_rx)),
            subbed_tx,
            pods: Arc::new(DashMap::new()),
        })
    }

    pub fn get_hosts(&self) -> ServiceResult<Vec<String>> {
        let mut conn = self.client.get_connection()?;
        let hosts: Vec<String> = conn.get("hosts")?;
        Ok(hosts)
    }

    pub fn is_subbed(&self, service_name: &String, method_name: &String) -> bool {
        self.subbed
            .contains(&format!("service_{}_{}", service_name, method_name))
    }

    pub fn probe_cache(
        &self,
        service_name: &String,
        method_name: &String,
        primary_key: &Value,
    ) -> ServiceResult<Option<bytes::Bytes>> {
        let name = format!("{}_{}", service_name, method_name);
        let val = self.get_entry(&name)?;
        let message = val.value();
        message.check_cache(primary_key)
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
        Ok(RemoteSender {
            addr,
            method_fields_map: self.method_fields_map.clone(),
        })
    }

    pub async fn sub_service(
        &self,
        service_name: &String,
        method_name: &String,
    ) -> ServiceResult<()> {
        use redis::AsyncCommands;
        let mut conn = self.client.get_async_connection().await?;

        let method: Method = {
            let rv: Vec<u8> = conn
                .get(format!("config_{}_{}", service_name, method_name))
                .await?;
            log::info!(
                "parsing sub_service config for {}_{}",
                service_name,
                method_name
            );
            serde_json::from_slice(&rv[..])?
        };

        log::info!(
            "querying messages config for {}_{}",
            service_name,
            method_name
        );
        let message = {
            let parents = {
                let keys: Vec<String> = conn.keys(format!("message_{}_*", service_name)).await?;
                let out = Arc::new(DashMap::<String, Message>::new());
                for key in keys {
                    let rv: Vec<u8> = conn.get(&key).await?;
                    let mut message: Message = serde_json::from_slice(&rv[..])?;
                    message.parent = out.clone();
                    let key = key
                        .rsplit("_")
                        .next()
                        .ok_or("could not parse message name")?;
                    out.insert(key.to_string(), message);
                }
                out
            };
            let val = parents.get(&method.output_message);
            let val = val.ok_or("no output message")?;
            val.value().clone()
        };

        log::info!("querying default cache config for {}", service_name,);
        let default_cache: CacheOptions = {
            let rv: Vec<u8> = conn.get(format!("default_{}", service_name)).await?;
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
            let mut pubsub = conn.into_pubsub();
            pubsub.subscribe(sub_name).await?;
        }
        self.subbed_tx.send(()).await?;
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
        log::info!("new cache published for: {}", name);
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
        log::info!("ready to receive updates from redis");
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
                let mut subbed_rx = self.subbed_rx.write().await;
                msg = tokio::select! {
                    v = on_msg.next() => match v {
                        Some(v) => v,
                        None => return Ok(()),
                    },
                    _ = subbed_rx.recv() => {
                        log::info!("new service added to cache listener, refreshing...");
                        continue;
                    }
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

    pub fn add_pod_to_watch(&self, pod_name: &String, service_name: &String) {
        self.pods.insert(pod_name.clone(), service_name.clone());
    }

    pub async fn watch_pods(&self, server: Arc<IntraServer>) -> ServiceResult<()> {
        use futures::prelude::*;
        let client = kube::Client::try_default().await?;
        let api: kube::Api<Pod> = kube::Api::default_namespaced(client);
        let pods = &self.pods.clone();

        let server = &server.clone();

        try_flatten_applied(watcher(api, Default::default()))
            .try_for_each(|p| async move {
                let name = p.metadata.name.unwrap();
                if !pods.contains_key(&name) {
                    return Ok(());
                }
                let status = p.status.as_ref().unwrap();
                if status.phase.as_ref().unwrap() == "Failed" {

                    // Remove from broker and server
                    let service = pods.get(&name).unwrap();
                    log::warn!(
                        "k8s: pod '{}', linked to service '{}' has been removed/evicted, announcing global listen...",
                        name,
                        service.value()
                    );
                    self.remove_service(service.value(), &name).await.unwrap();
                    {
                        server.remove_service(&name.to_string()).await;
                    }
                }
                Ok(())
            })
            .await?;
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
                        let client = {
                            let env = Arc::new(EnvBuilder::new().build());
                            let ch = ChannelBuilder::new(env).connect(host_addr.as_str());
                            api_proto::api_grpc::ApiClient::new(ch)
                        };
                        add_service_from_file(path, &Some(handler), &client)
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
        let mut cached = match self.method_fields_map.get(&name) {
            Some(v) => v.value().clone(),
            None => {
                return Err(ServiceError::new(
                    format!("received service fields with unknown name: {}", name).as_str(),
                ))
            }
        };
        let (primary_key, payload): (Value, Vec<u8>) =
            serde_json::from_slice(msg.get_payload_bytes())?;
        let fields_map = cached.fields_for_key.clone();
        let cached_fields = CachedFields {
            data: bytes::Bytes::copy_from_slice(&payload[..]),
            fields: cached.message.fields_from_bytes(&payload[..])?,
            timestamp: SystemTime::now(),
        };
        fields_map.insert(primary_key.clone(), cached_fields);
        cached.fields_for_key = fields_map;
        self.method_fields_map.insert(name.clone(), cached);
        log::info!(
            "cache updated for {}: primary key value = {:?}",
            &name,
            &primary_key
        );
        Ok(())
    }

    fn set_default(cfg: &mut config::Config) -> ServiceResult<()> {
        cfg.set_default("redis.address", "redis://127.0.0.1/".to_string())?;
        cfg.set_default("cluster.addresses", vec!["redis://127.0.0.1/"])?;
        Ok(())
    }
}
