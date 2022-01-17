use crate::proto;
use access_json::JSONQuery;
use config;
use dashmap::DashMap;
use jq_rs::{self, JqProgram};
use protobuf;
use protobuf_parse;
use serde_protobuf;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::pin::Pin;
use std::str;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum Protocol {
    None,
    HTTP,
}

pub mod http {
    pub use crate::proto::http::api;
    pub use crate::proto::http::API;
}

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
    pub fn new(err: &str) -> Self {
        ServiceError {
            err: String::from_str(err).unwrap(),
        }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to create service object: {}", self.err)
    }
}

pub struct Value {
    pub value: String,
}

impl Value {
    fn new(val: String) -> Self {
        Self { value: val }
    }
}

pub trait Handler {
    fn from_payload(&self, buf: &[u8]) -> ServiceResult<Value>;
    fn from_proto(&self, message: protobuf::descriptor::DescriptorProto) -> ServiceResult<Value>;
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

pub struct Message {
    path: String,
    message: protobuf::descriptor::DescriptorProto,
}

impl Message {
    fn new(message: protobuf::descriptor::DescriptorProto, path: String) -> Self {
        Self {
            path: Default::default(),
            message,
        }
    }

    pub fn from_bytes(
        &self,
        buf: &[u8],
    ) -> protobuf::ProtobufResult<protobuf::descriptor::DescriptorProto> {
        let buf = protobuf::CodedInputStream::from_bytes(buf);
        let mut message = self.message.clone();
        message.merge_from_dyn(&mut buf)?;
        Ok(message)
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            message: self.message.clone(),
        }
    }
}

pub struct Method {
    pub api: MethodAPI,
    pub handler: Pin<Box<dyn Handler + Sync + Send + 'static>>,
    pub input_message: String,
    pub output_message: String,
}

pub type Services = DashMap<String, Service>;

pub struct Service {
    pub name: String,
    pub protocol: Protocol,
    pub methods: DashMap<String, Method>,
    pub messages: DashMap<String, Message>,
}

impl Service {
    pub fn from_file(path: &str) -> Result<Self, ServiceError> {
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

        let mut output = Self::default();
        output.get_service_attrs_base(file)?;
        match Self::get_service_type(service) {
            Protocol::HTTP => output.get_service_attrs_http(service)?,
            _ => panic!("unknown protocol"),
        };

        Ok(output)
    }

    pub fn from_config(cfg: config::Config) -> Result<Services, ServiceError> {
        let services = cfg.get_table("service").unwrap();
        Ok(services
            .iter()
            .map(|(name, value)| (name.clone(), Self::service_from_config_value(name, value)))
            .collect())
    }

    fn service_from_config_value(name: &String, value: &config::Value) -> Self {
        let service = value.into_table().unwrap();
        let proto = {
            let p = service.get("proto").unwrap();
            let p = p.into_str().unwrap();
            p
        };
        Self::from_file(proto.as_str()).unwrap()
    }

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
        &mut self,
        file: &protobuf::descriptor::FileDescriptorProto,
    ) -> Result<(), ServiceError> {
        use proto::pandit::exts;
        self.messages = file
            .message_type
            .iter()
            .map(|message| {
                let name = message.get_name().to_string();
                let opts = message.options.get_ref();
                let path = exts::path.get(opts).unwrap();
                let mut config = Message::new(message.clone(), path);
                (name, config)
            })
            .collect();
        Ok(())
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
                (
                    method.get_name().to_string(),
                    Method {
                        input_message: method.get_input_type().to_string(),
                        output_message: method.get_output_type().to_string(),
                        handler: Service::handler_from_http_api(api),
                        api: MethodAPI {
                            http: ManuallyDrop::new(api),
                        },
                    },
                )
            })
            .collect();

        Ok(())
    }

    fn handler_from_http_api(api: http::API) -> Pin<Box<dyn Handler + Sync + Send + 'static>> {
        match api.content_type.as_str() {
            "application/json" => Box::pin(HttpJsonHandler::new(api.pattern.unwrap())),
        }
    }
}

pub struct HttpJsonHandler {
    pub method: http::api::Pattern,
    prog: JSONQuery,
    path: String,
}

impl HttpJsonHandler {
    pub fn new(method: http::api::Pattern) -> Self {
        let prog = match method {
            http::api::Pattern::get(x) => x,
            http::api::Pattern::put(x) => x,
            http::api::Pattern::post(x) => x,
            http::api::Pattern::delete(x) => x,
            http::api::Pattern::patch(x) => x,
        };
        let path = prog.to_string();
        Self {
            method,
            prog: JSONQuery::parse(path.as_str()).unwrap(),
            path,
        }
    }
}

impl Handler for HttpJsonHandler {
    fn from_payload(&self, buf: &[u8]) -> ServiceResult<Value> {
        let json: serde_json::Value = serde_json::from_slice(buf).unwrap();
        let pr = self.prog.execute(&json).unwrap();
        let result = pr.unwrap();
        match result.as_str() {
            Some(v) => Ok(Value::new(String::from_str(v).unwrap())),
            None => Err(ServiceError::new(
                "result was unable to be serialized into String",
            )),
        }
    }

    fn from_proto(&self, message: protobuf::descriptor::DescriptorProto) -> ServiceResult<Value> {
        use serde_protobuf::descriptor::MessageDescriptor;
        let message = MessageDescriptor::from_proto("", &message);
        let value = self.prog.execute(&message).unwrap();
        let result = value.unwrap();
        match result.as_str() {
            Some(v) => Ok(Value::new(String::from_str(v).unwrap())),
            None => Err(ServiceError::new(
                "result was unable to be serialized into String",
            )),
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self {
            name: Default::default(),
            methods: Default::default(),
            messages: Default::default(),
            protocol: Protocol::None,
        }
    }
}

pub fn new_config(path: &str) -> config::Config {
    let mut obj = config::Config::new();
    let file = config::File::from(PathBuf::from(path)).format(config::FileFormat::Yaml);
    obj.merge(file).unwrap();
    obj
}

#[test]
fn test_service() {
    let s = Service::from_file("./src/proto/example.proto").unwrap();
    assert_eq!(s.protocol, Protocol::HTTP);
    assert_eq!(s.messages.len(), 2);
    assert_eq!(s.methods.len(), 1);
}
