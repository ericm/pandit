mod message;
mod value;

use crate::handlers::json::JsonHandler;
use crate::proto;
use crate::services::message::Message;
use access_json::JSONQuery;
use async_trait::async_trait;
use config;
use dashmap::DashMap;
use protobuf;
use protobuf_parse;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::str;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt::Display, path::PathBuf};
use tokio::sync::Mutex;
use value::Value;

#[derive(Debug, PartialEq)]
pub enum Protocol {
    None,
    HTTP,
}

pub mod http {
    pub use crate::proto::http::api;
    pub use crate::proto::http::API;
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
    pub http: ManuallyDrop<http::API>,
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
    map: FieldsMap,
}

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
    pub handler: Box<dyn Handler + Sync + Send + 'static>,
    pub input_message: String,
    pub output_message: String,
}

pub type Services = DashMap<String, Service>;

type WriterRef = Box<Mutex<dyn Writer>>;

pub struct Service {
    pub name: String,
    pub protocol: Protocol,
    pub methods: DashMap<String, Method>,
    pub messages: Arc<DashMap<String, Message>>,
    pub writer: WriterRef,
}

impl Service {
    pub fn from_file(path: &str, writer: WriterRef) -> Result<Self, ServiceError> {
        let path_buf = &PathBuf::from(path);
        let include = PathBuf::from("./src/proto");
        let parsed =
            protobuf_parse::pure::parse_and_typecheck(&[include], &[path_buf.clone()]).unwrap();
        let filename = path_buf.file_name().unwrap();
        let file = parsed
            .file_descriptors
            .iter()
            .find(|&x| x.get_name() == filename)
            .unwrap();
        let service = file.service.first().unwrap();

        let mut output = Self::get_service_attrs_base(file, writer)?;
        match Self::get_service_type(service) {
            Protocol::HTTP => output.get_service_attrs_http(service)?,
            _ => panic!("unknown protocol"),
        };

        Ok(output)
    }

    // pub fn from_config(cfg: config::Config) -> Result<Services<'w>, ServiceError> {
    //     let services = cfg.get_table("service").unwrap();
    //     Ok(services
    //         .iter()
    //         .map(|(name, value)| {
    //             (
    //                 name.clone(),
    //                 Self::service_from_config_value(name, value.clone()),
    //             )
    //         })
    //         .collect())
    // }

    // fn service_from_config_value(name: &String, value: config::Value) -> Self {
    //     let service = value.into_table().unwrap();
    //     let proto = {
    //         let p = service.get("proto").unwrap().clone();
    //         let p = p.into_str().unwrap();
    //         p
    //     };
    //     Self::from_file(proto.as_str()).unwrap()
    // }

    fn get_service_type(service: &protobuf::descriptor::ServiceDescriptorProto) -> Protocol {
        if proto::http::exts::name
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
    ) -> Result<Self, ServiceError> {
        use proto::pandit::exts;
        let mut messages: Arc<DashMap<String, Message>> = Arc::new(DashMap::new());
        messages = Arc::new(
            file.message_type
                .iter()
                .map(|message| {
                    let name = message.get_name().to_string();
                    println!("{}", name);
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
        Ok(Self {
            name: Default::default(),
            methods: Default::default(),
            protocol: Protocol::None,
            messages,
            writer,
        })
    }

    fn get_service_attrs_http(
        &mut self,
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<(), ServiceError> {
        use proto::http::exts;

        let opts = service.options.get_ref();
        self.name = exts::name.get(opts).unwrap();
        self.protocol = Protocol::HTTP;

        self.methods = service
            .method
            .iter()
            .map(|method| {
                let api = exts::api.get(method.options.get_ref()).unwrap();
                let input_message = method.get_input_type().to_string();
                let input_message = input_message.split('.').last().unwrap().to_string();
                let output_message = method.get_output_type().to_string();
                let output_message = output_message.split('.').last().unwrap().to_string();
                (
                    method.get_name().to_string(),
                    Method {
                        input_message: input_message.clone(),
                        output_message: output_message.clone(),
                        handler: self.handler_from_http_api(&output_message, api.clone()),
                        api: MethodAPI {
                            http: ManuallyDrop::new(api),
                        },
                    },
                )
            })
            .collect();

        Ok(())
    }

    fn handler_from_http_api(
        &self,
        message_name: &String,
        api: http::API,
    ) -> Box<dyn Handler + Sync + Send + 'static> {
        println!("n{}", *message_name);
        let message = self.messages.get(message_name).unwrap();
        match api.content_type.as_str() {
            "application/json" => {
                Box::new(JsonHandler::new(api.pattern.unwrap(), message.path.clone()))
            }
            e => panic!("unknown http api content type: {}", e),
        }
    }

    pub async fn send_proto_to_local(
        &mut self,
        method: &String,
        data: &[u8],
    ) -> ServiceResult<bytes::Bytes> {
        let method = self.methods.get_mut(method).unwrap();
        let messages = self.messages.clone();
        let message = messages.get(&method.input_message).unwrap();

        let fields = message.fields_from_bytes(data)?;
        let writer = self.writer.get_mut();
        let context = Self::context_from_api(&method.api)?;

        let resp = writer
            .write_request(context, &fields, &method.handler)
            .await?;

        let resp_fields = method.handler.from_payload(resp)?;
        let buf: Vec<u8> = Vec::with_capacity(1000);
        use bytes::BufMut;
        let mut buf = buf.writer();
        {
            let mut output = protobuf::CodedOutputStream::new(&mut buf);
            message.write_bytes_from_fields(&mut output, &resp_fields)?;
        }
        let buf = buf.into_inner();
        Ok(bytes::Bytes::copy_from_slice(&buf[..]))
    }

    fn context_from_api(api: &MethodAPI) -> ServiceResult<WriterContext> {
        let context = WriterContext::new();
        match unsafe { api.http.pattern.as_ref() }.ok_or(ServiceError::new("no pattern in api"))? {
            http::api::Pattern::get(s) => {
                context.insert("method".to_string(), "GET".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            http::api::Pattern::put(s) => {
                context.insert("method".to_string(), "PUT".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            http::api::Pattern::post(s) => {
                context.insert("method".to_string(), "POST".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            http::api::Pattern::delete(s) => {
                context.insert("method".to_string(), "DELETE".to_string());
                context.insert("uri".to_string(), s.clone());
            }
            http::api::Pattern::patch(s) => {
                context.insert("method".to_string(), "PATCH".to_string());
                context.insert("uri".to_string(), s.clone());
            }
        }
        Ok(context)
    }
}

pub type WriterContext = DashMap<String, String>;

#[async_trait]
pub trait Writer: Sync + Send {
    async fn write_request(
        &mut self,
        context: WriterContext,
        fields: &Fields,
        handler: &Box<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes>;
}

pub fn new_config(path: &str) -> config::Config {
    let mut obj = config::Config::new();
    let file = config::File::from(PathBuf::from(path)).format(config::FileFormat::Yaml);
    obj.merge(file).unwrap();
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
            handler: &Box<dyn Handler + Send + Sync>,
        ) -> ServiceResult<bytes::Bytes> {
            self.context = Some(context);
            self.fields = Some(fields.clone());
            Ok(bytes::Bytes::from_static(b"{\"id\": 1}"))
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
        let mut service = Service::from_file("./src/proto/example.proto", writer_ref).unwrap();
        let buf: &[u8] = &[
            0x08, 0x96, 0x01, // Field varint
        ];
        let resp = service
            .send_proto_to_local(&"GetExample".to_string(), buf)
            .await
            .unwrap();
        assert_eq!(
            resp,
            bytes::Bytes::from_static(&[
                0x08, 0x96, 0x01, // Field varint
            ])
        );
    }
}
