use crate::proto;
use ::std::slice;
use access_json::{AnySerializable, JSONQuery};
use config;
use dashmap::mapref::one::Ref;
use dashmap::{DashMap, DashSet};
use jq_rs::{self, JqProgram};
use protobuf::descriptor::FieldDescriptorProto;
use protobuf::reflect::runtime_types::*;
use protobuf::reflect::ProtobufValue;
use protobuf::reflect::ReflectValueBox;
use protobuf::reflect::ReflectValueRef;
use protobuf::reflect::RuntimeTypeBox;
use protobuf::wire_format::Tag;
use protobuf::{self, CodedInputStream, MessageDyn, ProtobufResult};
use protobuf_parse;
use serde::ser::SerializeMap;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
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

#[derive(Debug, Clone)]
pub enum Value {
    String(Vec<String>),
    Bytes(Vec<Vec<u8>>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    UInt32(Vec<u32>),
    UInt64(Vec<u64>),
    Float64(Vec<f64>),
    Float32(Vec<f32>),
    Bool(Vec<bool>),
    Enum(Vec<ProtoEnum>),
    Message(Vec<(String, FieldsMap)>),
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

    pub fn from_message(name: String, fields: FieldsMap) -> Self {
        Self::Message(vec![(name, fields)])
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, sr: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        fn serialize_seq<S, T>(
            sr: S,
            v: &Vec<T>,
        ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
        where
            S: serde::Serializer,
            T: Serialize,
        {
            use serde::ser::SerializeSeq;
            let mut seq = sr.serialize_seq(Some(v.len()))?;
            for item in v {
                seq.serialize_element(item).unwrap();
            }
            seq.end()
        }
        match self {
            Value::String(v) => serialize_seq(sr, &v),
            Value::Bytes(v) => serialize_seq(sr, &v),
            Value::Int32(v) => serialize_seq(sr, &v),
            Value::Int64(v) => serialize_seq(sr, &v),
            Value::UInt32(v) => serialize_seq(sr, &v),
            Value::UInt64(v) => serialize_seq(sr, &v),
            Value::Float64(v) => serialize_seq(sr, &v),
            Value::Float32(v) => serialize_seq(sr, &v),
            Value::Bool(v) => serialize_seq(sr, &v),
            Value::Enum(v) => todo!(),
            Value::Message(v) => todo!(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Bytes(l0), Self::Bytes(r0)) => l0 == r0,
            (Self::Int32(l0), Self::Int32(r0)) => l0 == r0,
            (Self::Int64(l0), Self::Int64(r0)) => l0 == r0,
            (Self::UInt32(l0), Self::UInt32(r0)) => l0 == r0,
            (Self::UInt64(l0), Self::UInt64(r0)) => l0 == r0,
            (Self::Float64(l0), Self::Float64(r0)) => l0 == r0,
            (Self::Float32(l0), Self::Float32(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Enum(l0), Self::Enum(r0)) => l0 == r0,
            (Self::Message(l0), Self::Message(r0)) => {
                l0.len() == r0.len()
                    && l0.iter().zip(r0).all(|(l, r)| {
                        l.0 == r.0
                            && l.1.len() == r.1.len()
                            && l.1.iter().all(|k| r.1.contains_key(k.key()))
                    })
            }
            _ => false,
        }
    }
}

pub trait Handler {
    fn from_payload(&self, buf: &[u8]) -> ServiceResult<Value>;
    // fn to_payload(&self, fields: &Fields) -> ServiceResult<Vec<u8>>;
    fn from_proto(&self, message: protobuf::descriptor::DescriptorProto) -> ServiceResult<Value>;
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

pub struct Fields {
    map: FieldsMap,
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

pub struct Message {
    path: String,
    fields: DashMap<u32, FieldDescriptorProto>,
    message: protobuf::descriptor::DescriptorProto,
    parent: Arc<DashMap<String, Message>>,
}

impl Message {
    fn new(
        message: protobuf::descriptor::DescriptorProto,
        path: String,
        parent: Arc<DashMap<String, Message>>,
    ) -> Self {
        let fields: DashMap<u32, FieldDescriptorProto> = message
            .field
            .iter()
            .map(|field| {
                let number = u32::try_from(field.get_number()).unwrap();
                (number, field.clone())
            })
            .collect();
        Self {
            path,
            fields,
            message,
            parent,
        }
    }

    pub fn fields_from_bytes(&self, buf: &[u8]) -> ServiceResult<FieldsMap> {
        use std::convert::TryInto;

        let mut input = protobuf::CodedInputStream::from_bytes(buf);
        self.fields_from_bytes_delimited(&mut input, buf.len().try_into()?)
    }

    fn fields_from_bytes_delimited(
        &self,
        input: &mut CodedInputStream,
        len: u64,
    ) -> ServiceResult<FieldsMap> {
        let fields = FieldsMap::new();
        while input.pos() < len && !input.eof()? {
            let (name, value) = match self.from_field_descriptor_proto(input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("soft proto parsing error: {:?}", e);
                    continue;
                }
            };
            fields.insert(name, value);
        }

        Ok(fields)
    }

    fn from_field_descriptor_proto(
        &self,
        input: &mut CodedInputStream,
    ) -> ServiceResult<(String, Option<Value>)> {
        use protobuf::descriptor::field_descriptor_proto::Type::*;
        let tag = input.read_tag()?;
        let (number, wire_type) = tag.unpack();
        let field = self.fields.get(&number).ok_or(ServiceError::new(
            format!("field unknown: number({})", number).as_str(),
        ))?;
        Ok((
            field.get_name().to_string(),
            match field.get_field_type() {
                TYPE_DOUBLE => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_double_into(wire_type, input, &mut target)?;
                    Some(Value::Float64(target))
                }
                TYPE_FLOAT => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_float_into(wire_type, input, &mut target)?;
                    Some(Value::Float32(target))
                }
                TYPE_INT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_int64_into(wire_type, input, &mut target)?;
                    Some(Value::Int64(target))
                }
                TYPE_UINT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_uint64_into(wire_type, input, &mut target)?;
                    Some(Value::UInt64(target))
                }
                TYPE_INT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_int32_into(wire_type, input, &mut target)?;
                    Some(Value::Int32(target))
                }
                TYPE_FIXED64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_fixed64_into(wire_type, input, &mut target)?;
                    Some(Value::UInt64(target))
                }
                TYPE_FIXED32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_fixed32_into(wire_type, input, &mut target)?;
                    Some(Value::UInt32(target))
                }
                TYPE_BOOL => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_bool_into(wire_type, input, &mut target)?;
                    Some(Value::Bool(target))
                }
                TYPE_STRING => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_string_into(wire_type, input, &mut target)?;
                    Some(Value::String(target))
                }
                TYPE_BYTES => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_bytes_into(wire_type, input, &mut target)?;
                    Some(Value::Bytes(target))
                }
                TYPE_UINT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_uint32_into(wire_type, input, &mut target)?;
                    Some(Value::UInt32(target))
                }
                TYPE_SFIXED32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sfixed32_into(wire_type, input, &mut target)?;
                    Some(Value::Int32(target))
                }
                TYPE_SFIXED64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sfixed64_into(wire_type, input, &mut target)?;
                    Some(Value::Int64(target))
                }
                TYPE_SINT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sint32_into(wire_type, input, &mut target)?;
                    Some(Value::Int32(target))
                }
                TYPE_SINT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sint64_into(wire_type, input, &mut target)?;
                    Some(Value::Int64(target))
                }
                TYPE_ENUM => {
                    let mut target: Vec<ProtoEnum> = Vec::new();
                    protobuf::rt::read_repeated_enum_into(wire_type, input, &mut target)?;
                    Some(Value::Enum(target))
                }
                TYPE_MESSAGE => {
                    let message_name = field.get_type_name().to_string();
                    match self.parse_another_message(input, &message_name, field) {
                        Ok(v) => Some(Value::Message(v)),
                        _ => None,
                    }
                }
                _ => None,
            },
        ))
    }

    fn parse_another_message(
        &self,
        input: &mut CodedInputStream,
        message_name: &String,
        field: Ref<u32, FieldDescriptorProto>,
    ) -> ServiceResult<Vec<(String, FieldsMap)>> {
        use protobuf::descriptor::field_descriptor_proto::Label::*;
        let parent = self.parent.clone();
        let message = parent.get(message_name).ok_or(ServiceError::new(
            format!("no message called: {}", message_name).as_str(),
        ))?;
        let repeated_len = if field.get_label() == LABEL_REPEATED {
            input.pos() + input.read_raw_varint64()?
        } else {
            input.pos() + 1
        };
        let mut output: Vec<(String, FieldsMap)> = Vec::with_capacity(2);
        while input.pos() < repeated_len && !input.eof()? {
            let len = input.pos() + input.read_raw_varint64()?;
            output.push((
                message_name.clone(),
                message.fields_from_bytes_delimited(input, len)?,
            ));
        }
        Ok(output)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ProtoEnum {
    Val(i32),
}

impl Default for ProtoEnum {
    fn default() -> Self {
        Self::Val(Default::default())
    }
}

impl ProtobufValue for ProtoEnum {
    type RuntimeType = RuntimeTypeEnum<ProtoEnum>;
}

impl protobuf::ProtobufEnum for ProtoEnum {
    fn value(&self) -> i32 {
        match self {
            Self::Val(i) => *i,
            _ => panic!("unknown error"),
        }
    }

    fn from_i32(v: i32) -> Option<Self> {
        Some(Self::Val(v))
    }

    fn values() -> &'static [Self] {
        static VALUES: &'static [ProtoEnum] = &[ProtoEnum::Val(1)];
        VALUES
    }
}

mod message_tests {
    use std::hash::Hash;

    use super::ServiceResult;

    #[test]
    fn test_fields_from_bytes() -> ServiceResult<()> {
        use super::*;
        use protobuf::descriptor::field_descriptor_proto::Label::{self, *};
        use protobuf::descriptor::field_descriptor_proto::Type::{self, *};

        let message2_fields: FieldsMap = DashMap::new();
        message2_fields.insert("varint".to_string(), Some(Value::from_int32(150)));
        struct Table {
            name: String,
            field_type: Type,
            number: i32,
            type_name: String,
            label: Label,
            want: Value,
        }
        let table = [
            Table {
                name: "varint".to_string(),
                field_type: TYPE_INT32,
                number: 1,
                type_name: "".to_string(),
                label: LABEL_OPTIONAL,
                want: Value::from_int32(150),
            },
            Table {
                name: "string".to_string(),
                field_type: TYPE_STRING,
                number: 2,
                type_name: "".to_string(),
                label: LABEL_OPTIONAL,
                want: Value::from_string("testing".to_string()),
            },
            Table {
                name: "message".to_string(),
                field_type: TYPE_MESSAGE,
                number: 3,
                type_name: "Message2".to_string(),
                label: LABEL_OPTIONAL,
                want: Value::from_message("Message2".to_string(), message2_fields.clone()),
            },
            Table {
                name: "message_repeated".to_string(),
                field_type: TYPE_MESSAGE,
                number: 4,
                type_name: "Message2".to_string(),
                label: LABEL_REPEATED,
                want: Value::Message(vec![
                    ("Message2".to_string(), message2_fields.clone()),
                    ("Message2".to_string(), message2_fields.clone()),
                ]),
            },
        ];

        let buf: &[u8] = &[
            0x08, 0x96, 0x01, // Field varint
            0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, // Field string
            0x1a, 0x03, 0x08, 0x96, 0x01, // Embedded message
            0x22, 0x08, 0x03, 0x08, 0x96, 0x01, 0x03, 0x08, 0x96,
            0x01, // Embedded message repeated x2
        ];

        let mut desc = protobuf::descriptor::DescriptorProto::new();
        for item in &table {
            let mut field = protobuf::descriptor::FieldDescriptorProto::new();
            field.set_name(item.name.clone());
            field.set_field_type(item.field_type);
            field.set_number(item.number);
            field.set_type_name(item.type_name.clone());
            field.set_label(item.label);
            desc.field.push(field);
        }

        let mut message2 = protobuf::descriptor::DescriptorProto::new();
        message2.field.push(desc.field.first().unwrap().clone());

        let parent: Arc<DashMap<String, Message>> = Arc::new(DashMap::new());
        parent.insert(
            "Message2".to_string(),
            Message::new(message2, "".to_string(), parent.clone()),
        );

        let m = Message::new(desc, "".to_string(), parent);
        let output = m.fields_from_bytes(buf)?;
        for item in &table {
            let output = output
                .get(&item.name)
                .ok_or(ServiceError::new("fields incorrect"))?
                .value()
                .clone()
                .ok_or(ServiceError::new("fields incorrect"))?;
            assert_eq!(output, item.want, "expected 150, got {:?}", output);
        }
        Ok(())
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            message: self.message.clone(),
            fields: self.fields.clone(),
            parent: self.parent.clone(),
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
    pub messages: Arc<DashMap<String, Message>>,
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
            .map(|(name, value)| {
                (
                    name.clone(),
                    Self::service_from_config_value(name, value.clone()),
                )
            })
            .collect())
    }

    fn service_from_config_value(name: &String, value: config::Value) -> Self {
        let service = value.into_table().unwrap();
        let proto = {
            let p = service.get("proto").unwrap().clone();
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
        self.messages = Arc::new(DashMap::new());
        self.messages = Arc::new(
            file.message_type
                .iter()
                .map(|message| {
                    let name = message.get_name().to_string();
                    let opts = message.options.get_ref();
                    let path = exts::path.get(opts).unwrap();
                    let mut config = Message::new(message.clone(), path, self.messages.clone());
                    (name, config)
                })
                .collect(),
        );
        self.messages
            .iter_mut()
            .for_each(|mut m| m.parent = self.messages.clone());
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
                        handler: Service::handler_from_http_api(api.clone()),
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
            e => panic!("unknown http api content type: {}", e),
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
        use proto::http::api::Pattern::*;
        let prog = match method {
            get(ref x) | put(ref x) | post(ref x) | delete(ref x) | patch(ref x) => x,
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
            Some(v) => Ok(Value::from_string(String::from_str(v).unwrap())),
            None => Err(ServiceError::new(
                "result was unable to be serialized into String",
            )),
        }
    }

    fn from_proto(&self, message: protobuf::descriptor::DescriptorProto) -> ServiceResult<Value> {
        let value = self.prog.execute(&message).unwrap();
        let result = value.unwrap();
        match result.as_str() {
            Some(v) => Ok(Value::from_string(String::from_str(v).unwrap())),
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
