use crate::proto;
use access_json::JSONQuery;
use config;
use dashmap::DashMap;
use jq_rs::{self, JqProgram};
use protobuf::descriptor::FieldDescriptorProto;
use protobuf::reflect::ProtobufValue;
use protobuf::{self, CodedInputStream, MessageDyn, ProtobufResult};
use protobuf_parse;
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

pub enum Value {
    String(Vec<String>),
    Int32(Vec<i32>),
    UInt32(Vec<u32>),
}

impl Value {
    pub fn from_string(val: String) -> Self {
        Self::String(vec![val])
    }

    pub fn from_int32(val: i32) -> Self {
        Self::Int32(vec![val])
    }

    pub fn from_uint32(val: u32) -> Self {
        Self::UInt32(vec![val])
    }

    fn from_field_descriptor_proto(
        input: &mut CodedInputStream,
        field: &FieldDescriptorProto,
    ) -> ProtobufResult<Self> {
        use protobuf::descriptor::field_descriptor_proto::Type::*;
        let (field_number, wire_type) = input.read_tag_unpack().unwrap();

        match field.get_field_type() {
            TYPE_DOUBLE => todo!(),
            TYPE_FLOAT => todo!(),
            TYPE_INT64 => todo!(),
            TYPE_UINT64 => todo!(),
            TYPE_INT32 => {
                let target = Vec::new();
                protobuf::rt::read_repeated_int32_into(wire_type, input, &mut target)?;
                Ok(Self::Int32(target))
            }
            TYPE_FIXED64 => todo!(),
            TYPE_FIXED32 => todo!(),
            TYPE_BOOL => todo!(),
            TYPE_STRING => {
                let target = Vec::new();
                protobuf::rt::read_repeated_string_into(wire_type, input, &mut target)?;
                Ok(Self::String(target))
            }
            TYPE_GROUP => todo!(),
            TYPE_MESSAGE => todo!(),
            TYPE_BYTES => todo!(),
            TYPE_UINT32 => todo!(),
            TYPE_ENUM => todo!(),
            TYPE_SFIXED32 => todo!(),
            TYPE_SFIXED64 => todo!(),
            TYPE_SINT32 => todo!(),
            TYPE_SINT64 => todo!(),
        }
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
        // let buf = protobuf::CodedInputStream::from_bytes(buf);
        // let buf = protobuf::well_known_types::Any::parse_from_bytes(buf).unwrap();
        let input = protobuf::CodedInputStream::from_bytes(buf);
        let target = protobuf::well_known_types::Any::default();

        let field_map: DashMap<String, Option<Value>> = self
            .message
            .field
            .iter()
            .map(|field| {
                (
                    String::from(field.get_name()),
                    match Value::from_field_descriptor_proto(&mut input, field) {
                        Ok(val) => Some(val),
                        Err(e) => {
                            println!("soft proto parsing error: {:?}", e);
                            None
                        }
                    },
                )
            })
            .collect();
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
