pub mod message;
pub mod value;

use crate::broker::Broker;
use crate::handlers::json::JsonHandler;
use crate::handlers::sql::SQLHandler;
use crate::proto;
use crate::proto::gen::format::postgres::exts::postgres;
use crate::services::message::Message;
use access_json::JSONQuery;
use async_trait::async_trait;
use config;
use dashmap::DashMap;
use protobuf::descriptor::{MethodDescriptorProto, MethodOptions};
use protobuf::{self};
use protobuf_parse;
use redis::ToRedisArgs;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::BinaryHeap;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::str;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt::Display, path::PathBuf};
use tokio::sync::{Mutex, RwLock};
use value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Protocol {
    None,
    HTTP,
}

pub mod format {
    pub use crate::proto::gen::format::http::exts::http as http_api;
    pub use crate::proto::gen::format::http::http;
    pub use crate::proto::gen::format::http::HTTP;
    pub use crate::proto::gen::handler::exts as handlers;
}

pub mod base {
    pub use crate::proto::gen::pandit::exts::cache as method_cache;
    pub use crate::proto::gen::pandit::exts::default_cache;
    pub use crate::proto::gen::pandit::exts::field_cache;
    pub use crate::proto::gen::pandit::CacheOptions;
}

pub type ServiceResult<T> = Result<T, Box<dyn std::error::Error>>;

impl FromStr for Protocol {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::HTTP),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceError {
    err: String,
}

impl ServiceError {
    pub fn new(err: &str) -> Box<Self> {
        Box::new(ServiceError {
            err: String::from_str(err).unwrap(),
        })
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to create service object: {}", self.err)
    }
}

impl std::error::Error for ServiceError {}

#[async_trait]
pub trait Handler {
    fn from_payload(&self, buf: bytes::Bytes) -> ServiceResult<Fields>;
    async fn to_payload(&self, fields: &Fields) -> ServiceResult<bytes::Bytes>;
    // fn fields_to_payload(&self, fields: &Fields) {
    //     for field in fields.iter() {
    //         let value = match field.value() {
    //             Some(v) => v,
    //             None => continue,
    //         };
    //     }
    // }
}

pub union MethodAPI {
    pub http: ManuallyDrop<format::HTTP>,
}

pub struct MessageField {
    proto: Box<protobuf::descriptor::FieldDescriptorProto>,
    absolute_path: String,
    relative_path: String,
}

impl Default for MessageField {
    fn default() -> Self {
        Self {
            proto: Default::default(),
            absolute_path: Default::default(),
            relative_path: Default::default(),
        }
    }
}

pub type FieldsMap = DashMap<String, Option<Value>>;

#[derive(Debug, Clone)]
pub struct Fields {
    pub map: FieldsMap,
}

impl std::hash::Hash for Fields {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let vals: BinaryHeap<Value> = self
            .map
            .iter()
            .filter_map(|entry| entry.value().clone())
            .collect();
        for val in vals.into_iter_sorted() {
            val.hash(state);
        }
    }
}

impl PartialEq for Fields {
    fn eq(&self, other: &Self) -> bool {
        let mut s_hash = DefaultHasher::new();
        self.hash(&mut s_hash);
        let mut o_hash = DefaultHasher::new();
        other.hash(&mut o_hash);
        s_hash.finish() == o_hash.finish()
    }
}

impl Eq for Fields {}

struct FieldsVisitor {}

impl<'de> Visitor<'de> for FieldsVisitor {
    type Value = Fields;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map of strings to values")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let map = FieldsMap::new();
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }
        Ok(Fields::new(map))
    }
}

impl<'de> Deserialize<'de> for Fields {
    fn deserialize<D>(dr: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = FieldsVisitor {};
        dr.deserialize_map(visitor)
    }
}

impl Fields {
    pub fn new(map: FieldsMap) -> Self {
        Self { map }
    }
}

impl Serialize for Fields {
    fn serialize<S>(
        &self,
        sr: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = sr.serialize_map(Some(self.map.len()))?;
        for kv in &self.map {
            match kv.value() {
                Some(value) => {
                    map.serialize_entry(kv.key(), value).unwrap();
                }
                None => continue,
            }
        }
        map.end()
    }
}

pub struct Method {
    pub api: MethodAPI,
    pub handler: Option<Arc<dyn Handler + Sync + Send + 'static>>,
    pub input_message: String,
    pub output_message: String,
    pub cache: Option<base::CacheOptions>,
    pub primary_key: Option<String>,
}

impl Serialize for base::CacheOptions {
    fn serialize<S>(&self, sr: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut strc = sr.serialize_struct("CacheOptions", 2)?;
        strc.serialize_field("disable", &self.disable)?;
        strc.serialize_field("cache_time", &self.cache_time)?;
        strc.end()
    }
}

impl<'de> Deserialize<'de> for base::CacheOptions {
    fn deserialize<D>(dr: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor {}
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = base::CacheOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("CacheOptions")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut disable = None;
                let mut cache_time = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "disable" => {
                            disable = Some(map.next_value()?);
                        }
                        "cache_time" => {
                            cache_time = Some(map.next_value()?);
                        }
                        _ => continue,
                    }
                }
                let mut out = base::CacheOptions::new();
                out.disable = disable.ok_or_else(|| de::Error::missing_field("disable"))?;
                out.cache_time =
                    cache_time.ok_or_else(|| de::Error::missing_field("cache_time"))?;
                Ok(out)
            }
        }

        let vis = ValueVisitor {};
        dr.deserialize_struct("CacheOptions", &["disable", "cache_time"], vis)
    }
}

impl Serialize for Method {
    fn serialize<S>(&self, sr: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut strc = sr.serialize_struct("Method", 4)?;
        strc.serialize_field("input_message", &self.input_message)?;
        strc.serialize_field("output_message", &self.output_message)?;
        strc.serialize_field("cache", &self.cache)?;
        strc.serialize_field("primary_key", &self.primary_key)?;
        strc.end()
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(dr: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor {}
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Method;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Method")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut input_message = None;
                let mut output_message = None;
                let mut cache = None;
                let mut primary_key = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "input_message" => {
                            input_message = Some(map.next_value()?);
                        }
                        "output_message" => {
                            output_message = Some(map.next_value()?);
                        }
                        "cache" => {
                            cache = Some(map.next_value()?);
                        }
                        "primary_key" => {
                            primary_key = Some(map.next_value()?);
                        }
                        _ => continue,
                    }
                }
                let out = Method {
                    input_message: input_message
                        .ok_or_else(|| de::Error::missing_field("input_message"))?,
                    output_message: output_message
                        .ok_or_else(|| de::Error::missing_field("output_message"))?,
                    cache: cache.ok_or_else(|| de::Error::missing_field("cache"))?,
                    primary_key: primary_key
                        .ok_or_else(|| de::Error::missing_field("primary_key"))?,
                    api: MethodAPI {
                        http: Default::default(),
                    },
                    handler: None,
                };
                Ok(out)
            }
        }

        let vis = ValueVisitor {};
        dr.deserialize_struct("CacheOptions", &["disable", "cache_time"], vis)
    }
}

pub type Services = DashMap<String, Service>;

pub type WriterRef = Box<Mutex<dyn Writer>>;

pub struct Service {
    pub name: String,
    pub protocol: Protocol,
    pub methods: DashMap<String, Method>,
    pub messages: Arc<DashMap<String, Message>>,
    pub writer: WriterRef,
    pub default_handler: Option<Arc<dyn Handler + Sync + Send + 'static>>,
    pub default_cache: base::CacheOptions,
    pub broker: Arc<Broker>,
}

impl Service {
    pub fn from_file(
        path: &str,
        include: &[&str],
        writer: WriterRef,
        broker: Arc<Broker>,
    ) -> Result<Self, ServiceError> {
        let path_buf = &PathBuf::from(path);
        let include: Vec<PathBuf> = include.iter().map(|v| PathBuf::from(v)).collect();
        let parsed =
            protobuf_parse::pure::parse_and_typecheck(&include[..], &[path_buf.clone()]).unwrap();
        let filename = path_buf.file_name().unwrap().to_str().unwrap();
        let file = parsed
            .file_descriptors
            .iter()
            .find(|&x| {
                let name = x.get_name().to_string();
                name.ends_with(filename)
            })
            .unwrap();
        let service = file.service.first().unwrap();

        let mut output = Self::get_service_attrs_base(file, writer, broker, &service)?;
        match Self::get_service_type(&service) {
            Protocol::HTTP => output.get_service_attrs_http(&service)?,
            _ => panic!("unknown protocol"),
        };

        Ok(output)
    }

    fn get_service_type(service: &protobuf::descriptor::ServiceDescriptorProto) -> Protocol {
        if crate::proto::gen::pandit::exts::name
            .get(service.options.get_ref())
            .is_some()
        {
            Protocol::HTTP
        } else {
            Protocol::None
        }
    }

    fn get_service_attrs_base(
        file: &protobuf::descriptor::FileDescriptorProto,
        writer: WriterRef,
        broker: Arc<Broker>,
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<Self, ServiceError> {
        use proto::gen::pandit::exts;
        let mut messages: Arc<DashMap<String, Message>> = Arc::new(DashMap::new());
        messages = Arc::new(
            file.message_type
                .iter()
                .map(|message| {
                    let name = message.get_name().to_string();
                    log::info!("{}", name);
                    let opts = message.options.get_ref();
                    let path = exts::path.get(opts).unwrap_or("".to_string());
                    let config = Message::new(message.clone(), path, messages.clone());
                    (name, config)
                })
                .collect(),
        );
        messages
            .iter_mut()
            .for_each(|mut m| m.parent = messages.clone());

        let default_handler = {
            let handler = format::handlers::default_handler.get(service.options.get_ref());
            match handler {
                Some(v) => {
                    let val = v.enum_value().unwrap();
                    Self::handler(val, ".".to_string())
                }
                None => None,
            }
        };

        let default_cache = base::default_cache
            .get(service.options.get_ref())
            .unwrap_or_default();

        Ok(Self {
            name: Default::default(),
            methods: Default::default(),
            protocol: Protocol::None,
            broker,
            messages,
            writer,
            default_handler,
            default_cache,
        })
    }

    fn get_service_attrs_http(
        &mut self,
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<(), ServiceError> {
        use proto::gen::pandit::exts;

        let opts = service.options.get_ref();
        self.name = exts::name.get(opts).unwrap();
        self.protocol = Protocol::HTTP;

        self.methods = service
            .method
            .iter()
            .map(|method| {
                let api = format::http_api.get(method.options.get_ref()).unwrap();
                let input_message = method.get_input_type().to_string();
                let input_message = input_message.split('.').last().unwrap().to_string();
                let output_message = method.get_output_type().to_string();
                let output_message = output_message.split('.').last().unwrap().to_string();
                (
                    method.get_name().to_string(),
                    Method {
                        input_message: input_message.clone(),
                        output_message: output_message.clone(),
                        handler: self.handler_for_method(&method),
                        api: MethodAPI {
                            http: ManuallyDrop::new(api),
                        },
                        cache: base::method_cache.get(method.options.get_ref()),
                        primary_key: self.primary_key_for_method(&input_message),
                    },
                )
            })
            .collect();

        Ok(())
    }

    fn primary_key_for_method(&self, message_name: &String) -> Option<String> {
        let message = self.messages.get(message_name).unwrap();
        let message = message.value();
        for entry in message.fields_by_name.iter() {
            let opts = entry.value().descriptor.options.get_ref();
            let is_key = proto::gen::pandit::exts::key.get(opts);
            if is_key.unwrap_or_default() {
                return Some(entry.key().clone());
            }
        }
        None
    }

    fn handler_for_method(
        &self,
        method: &MethodDescriptorProto,
    ) -> Option<Arc<dyn Handler + Sync + Send + 'static>> {
        let message = {
            let name = method.get_output_type().to_string();
            let name = name.split('.').last().unwrap().to_string();
            self.messages.get(&name).unwrap()
        };
        let options = method.options.get_ref();
        {
            use format::handlers;
            match handlers::handler.get(options) {
                Some(val) => {
                    let val = val.enum_value().unwrap();
                    match Self::handler(val, message.path.clone()) {
                        Some(v) => Some(v),
                        None => {
                            // Quick solution to default postgres service to SQL handler.
                            match postgres.get(options) {
                                Some(opts) => {
                                    let input_message = method.get_input_type().to_string();
                                    let input_message =
                                        input_message.split('.').last().unwrap().to_string();
                                    let output_message = method.get_output_type().to_string();
                                    let output_message =
                                        output_message.split('.').last().unwrap().to_string();
                                    Some(Arc::new(SQLHandler::new(
                                        self.messages.clone(),
                                        input_message,
                                        output_message,
                                        opts,
                                    )))
                                }
                                None => None,
                            }
                        }
                    }
                }
                None => None,
            }
        }
    }

    fn handler(
        handler: proto::gen::handler::Handler,
        path: String,
    ) -> Option<Arc<dyn Handler + Sync + Send + 'static>> {
        use proto::gen::handler::Handler::*;
        match handler {
            JSON => Some(Arc::new(JsonHandler::new(path))),
            _ => None,
        }
    }

    fn context_from_api(api: &MethodAPI) -> ServiceResult<WriterContext> {
        let context = WriterContext::new();
        use crate::proto::gen::format::http::http::Pattern;
        match unsafe { api.http.pattern.as_ref() }.ok_or(ServiceError::new("no pattern in api"))? {
            Pattern::get(s) => {
                context.insert("method".to_string(), "GET".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            Pattern::put(s) => {
                context.insert("method".to_string(), "PUT".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            Pattern::post(s) => {
                context.insert("method".to_string(), "POST".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            Pattern::delete(s) => {
                context.insert("method".to_string(), "DELETE".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            Pattern::patch(s) => {
                context.insert("method".to_string(), "PATCH".to_string());
                context.insert("uri".to_string(), s.clone());
            }
        }
        Ok(context)
    }
}

#[async_trait]
pub trait Sender: Send + Sync {
    async fn send(
        &mut self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<bytes::Bytes>;

    async fn probe_cache(
        &self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<Option<bytes::Bytes>>;
}

#[async_trait]
impl Sender for Service {
    async fn send(
        &mut self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<bytes::Bytes> {
        let method = self.methods.get_mut(method).unwrap();
        let messages = self.messages.clone();
        let message = messages.get(&method.input_message).unwrap();
        let fields = message.fields_from_bytes(data)?;

        log::info!(
            "sending local data request for {}_{}",
            service_name,
            method.key()
        );

        let writer = self.writer.get_mut();
        let context = Self::context_from_api(&method.api)?;

        let handler = method
            .handler
            .as_ref()
            .or(self.default_handler.as_ref())
            .ok_or(ServiceError::new(
                format!(
                    "unable to find handler or default handler for {}.{}",
                    service_name,
                    method.key()
                )
                .as_str(),
            ))?;

        let resp = writer.write_request(context, &fields, handler).await?;
        let resp_fields = handler.from_payload(resp)?;

        let buf: Vec<u8> = Vec::with_capacity(1000);
        use bytes::BufMut;
        let mut buf = buf.writer();
        {
            let mut output = protobuf::CodedOutputStream::new(&mut buf);
            message.write_bytes_from_fields(&mut output, &resp_fields)?;
        }
        {
            self.broker
                .publish_cache(&service_name, method.key(), resp_fields)
                .await?;
        }

        let buf = buf.into_inner();
        Ok(bytes::Bytes::copy_from_slice(&buf[..]))
    }

    async fn probe_cache(
        &self,
        service_name: &String,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<Option<bytes::Bytes>> {
        let method = self.methods.get_mut(method).unwrap();
        let messages = self.messages.clone();
        let message = messages.get(&method.input_message).unwrap();

        let fields = message.fields_from_bytes(data)?;

        let primary_key = {
            let key = method
                .primary_key
                .as_ref()
                .ok_or(ServiceError::new("no primary key for method"))?;
            let val = fields.map.get(key);
            let val = val.ok_or(ServiceError::new(
                format!("error finding entry for primary key: {}", key).as_str(),
            ))?;
            let val = val.value().to_owned();
            val.ok_or(ServiceError::new(
                format!("no entry for primary key: {}", key).as_str(),
            ))?
        };
        self.broker
            .probe_cache(&service_name, method.key(), &primary_key)
    }
}

pub type WriterContext = DashMap<String, String>;

#[async_trait]
pub trait Writer: Sync + Send {
    async fn write_request(
        &mut self,
        context: WriterContext,
        fields: &Fields,
        handler: &Arc<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes>;
}

pub fn new_config(path: &str) -> config::Config {
    let mut obj = config::Config::new();
    let file = config::File::from(PathBuf::from(path)).format(config::FileFormat::Yaml);
    match obj.merge(file) {
        Ok(_) => {}
        Err(_) => {
            log::error!("warning: no config file provided");
            return Default::default();
        }
    }
    obj
}

mod tests {
    use super::*;
    struct FakeWriter {
        context: Option<WriterContext>,
        fields: Option<Fields>,
    }

    #[async_trait]
    impl Writer for FakeWriter {
        async fn write_request(
            &mut self,
            context: WriterContext,
            fields: &Fields,
            handler: &Arc<dyn Handler + Send + Sync>,
        ) -> ServiceResult<bytes::Bytes> {
            self.context = Some(context);
            self.fields = Some(fields.clone());
            Ok(bytes::Bytes::from_static(b"{\"obj\":{\"id\": 1}}"))
        }
    }

    #[tokio::test]
    async fn test_send_proto_to_local_http_json() {
        use super::*;
        let writer = FakeWriter {
            context: None,
            fields: None,
        };
        let writer_ref = Box::new(Mutex::new(writer));
        let broker = Broker::connect(Default::default(), "".to_string()).unwrap();
        let broker = Arc::new(broker);
        let mut service = Service::from_file(
            "./src/proto/examples/example1.proto",
            &["./src/proto"],
            writer_ref,
            broker.clone(),
        )
        .unwrap();
        broker
            .sub_service(&"ExampleService".to_string(), &"GetExample".to_string())
            .await
            .unwrap();
        let buf: &[u8] = &[
            0, 0, 0, 0, 0, // gRPC header.
            0x08, 0x96, 0x01, // Field varint
        ];
        let resp = service
            .send(
                &"ExampleService".to_string(),
                &"GetExample".to_string(),
                buf,
            )
            .await
            .unwrap();
        assert_eq!(
            resp,
            bytes::Bytes::from_static(&[
                0, 0, 0, 0, 0, // gRPC header.
                0x08, 0x01, // Field varint
            ])
        );
        return;
    }
}
