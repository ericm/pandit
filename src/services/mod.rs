use crate::proto;
use ::std::slice;
use access_json::{AnySerializable, JSONQuery};
use async_trait::async_trait;
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
use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::io::Write;
use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::ops::Deref;
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

#[derive(Debug, Clone, Deserialize)]
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
    Message(Vec<Fields>),
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

    pub fn from_message(fields: Fields) -> Self {
        Self::Message(vec![fields])
    }

    // pub fn from_query_value(value: )
}

struct EnumVisitor {}
impl<'de> Visitor<'de> for EnumVisitor {
    type Value = ProtoEnum;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("")
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProtoEnum::Val(v))
    }
}

impl<'de> Deserialize<'de> for ProtoEnum {
    fn deserialize<D>(dr: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = EnumVisitor {};
        dr.deserialize_i32(visitor)
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
            Value::Enum(v) => serialize_seq(sr, &v),
            Value::Message(v) => serialize_seq(sr, &v),
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
                        l.map.len() == r.map.len()
                            && l.map.iter().all(|k| r.map.contains_key(k.key()))
                    })
            }
            _ => false,
        }
    }
}

#[async_trait]
pub trait Handler {
    fn from_payload(&self, buf: bytes::Bytes) -> ServiceResult<Fields>;
    async fn to_payload_and_send(&mut self, fields: &Fields) -> ServiceResult<bytes::Bytes>;
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

pub struct Message {
    path: String,
    fields: DashMap<u32, FieldDescriptorProto>,
    fields_by_name: DashMap<String, FieldDescriptorProto>,
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
        let fields_by_name: DashMap<String, FieldDescriptorProto> = message
            .field
            .iter()
            .map(|field| {
                let name = field.get_name().to_string();
                (name, field.clone())
            })
            .collect();
        Self {
            path,
            fields,
            fields_by_name,
            message,
            parent,
        }
    }

    pub fn fields_from_bytes(&self, buf: &[u8]) -> ServiceResult<Fields> {
        use std::convert::TryInto;

        let mut input = protobuf::CodedInputStream::from_bytes(buf);
        self.fields_from_bytes_delimited(&mut input, buf.len().try_into()?)
    }

    fn fields_from_bytes_delimited(
        &self,
        input: &mut CodedInputStream,
        len: u64,
    ) -> ServiceResult<Fields> {
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

        Ok(Fields::new(fields))
    }

    pub fn write_bytes_from_fields(
        &self,
        output: &mut protobuf::CodedOutputStream,
        fields: &Fields,
    ) -> protobuf::ProtobufResult<()> {
        for kv in fields.map.iter() {
            let key = kv.key().clone();
            let value = match kv.value().clone() {
                Some(v) => v,
                None => {
                    eprintln!("no value for key {:?}", key);
                    continue;
                }
            };
            let field = self.fields_by_name.get(&key).ok_or(
                protobuf::ProtobufError::MessageNotInitialized(format!("no field: {}", key)),
            )?;

            use protobuf::descriptor::field_descriptor_proto::Label;
            use protobuf::descriptor::field_descriptor_proto::Type::*;
            fn write_value<'b, T: Clone>(
                field: Ref<String, FieldDescriptorProto>,
                output: &mut protobuf::CodedOutputStream<'b>,
                write: fn(&mut protobuf::CodedOutputStream<'b>, T) -> protobuf::ProtobufResult<()>,
                value: Vec<T>,
                wire_type: protobuf::wire_format::WireType,
            ) -> protobuf::ProtobufResult<()> {
                let num = u32::try_from(field.get_number()).unwrap();
                println!("write_value {} {}", field.get_name(), num);
                if field.get_label() == Label::LABEL_REPEATED {
                    output.write_tag(num, protobuf::wire_format::WireTypeLengthDelimited)?;
                    output.write_raw_varint64(u64::try_from(value.len()).unwrap())?;
                    for item in value {
                        write(output, item)?;
                    }
                    Ok(())
                } else {
                    let value = value
                        .first()
                        .ok_or(protobuf::ProtobufError::MessageNotInitialized(
                            "value does not exist".to_string(),
                        ))?
                        .clone();
                    output.write_tag(num, wire_type)?;
                    write(output, value)
                }
            }

            println!("match {}", field.get_name());
            match field.get_field_type() {
                TYPE_DOUBLE => {
                    let value = match value {
                        Value::Float64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_double_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed64,
                    )?;
                }
                TYPE_FLOAT => {
                    let value = match value {
                        Value::Float32(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_float_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed32,
                    )?;
                }
                TYPE_INT64 => {
                    let value = match value {
                        Value::Int64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_UINT64 => {
                    let value = match value {
                        Value::UInt64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_INT32 => {
                    let value = match value {
                        Value::Int32(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int32_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_FIXED64 => {
                    let value = match value {
                        Value::UInt64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed64,
                    )?;
                }
                TYPE_FIXED32 => {
                    let value = match value {
                        Value::UInt32(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed32_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed32,
                    )?;
                }
                TYPE_BOOL => {
                    let value = match value {
                        Value::Bool(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_bool_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_STRING => {
                    let value = match value {
                        Value::String(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        |output, s| {
                            protobuf::CodedOutputStream::write_string_no_tag(output, s.as_str())
                        },
                        value,
                        protobuf::wire_format::WireTypeLengthDelimited,
                    )?;
                }
                TYPE_BYTES => {
                    let value = match value {
                        Value::Bytes(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        |output, s| protobuf::CodedOutputStream::write_bytes_no_tag(output, &s[..]),
                        value,
                        protobuf::wire_format::WireTypeLengthDelimited,
                    )?;
                }
                TYPE_UINT32 => {
                    let value = match value {
                        Value::UInt64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_SFIXED32 => {
                    let value = match value {
                        Value::Int32(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed32_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed32,
                    )?;
                }
                TYPE_SFIXED64 => {
                    let value = match value {
                        Value::Int64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeFixed64,
                    )?;
                }
                TYPE_SINT32 => {
                    let value = match value {
                        Value::Int32(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint32_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_SINT64 => {
                    let value = match value {
                        Value::Int64(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint64_no_tag,
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_ENUM => {
                    let value = match value {
                        Value::Enum(v) => v,
                        _ => continue,
                    };
                    write_value(
                        field,
                        output,
                        |output, val| {
                            use protobuf::ProtobufEnum;
                            protobuf::CodedOutputStream::write_enum_no_tag(output, val.value())
                        },
                        value,
                        protobuf::wire_format::WireTypeVarint,
                    )?;
                }
                TYPE_MESSAGE => {
                    let message_name = field.get_type_name().to_string();
                    let parent = self.parent.clone();
                    let other_message = parent.get(&message_name).unwrap();
                    let value = match value {
                        Value::Message(v) => v,
                        _ => continue,
                    };

                    let num = u32::try_from(field.get_number()).unwrap();
                    output.write_tag(num, protobuf::wire_format::WireTypeLengthDelimited)?;

                    if field.get_label() == Label::LABEL_REPEATED {
                        let buf: Vec<u8> = Vec::with_capacity(10000);
                        use bytes::BufMut;
                        let mut buf = buf.writer();
                        {
                            let mut outer_output = protobuf::CodedOutputStream::new(&mut buf);
                            for item in value {
                                let buf: Vec<u8> = Vec::with_capacity(1000);
                                let mut buf = buf.writer();

                                {
                                    let mut sub_output = protobuf::CodedOutputStream::new(&mut buf);
                                    other_message
                                        .write_bytes_from_fields(&mut sub_output, &item)?;
                                }

                                let buf = buf.into_inner();
                                outer_output
                                    .write_raw_varint64(u64::try_from(buf.len()).unwrap())?;
                                outer_output.write_all(&buf[..])?;
                            }
                        }
                        let buf = buf.into_inner();
                        output.write_raw_varint64(u64::try_from(buf.len()).unwrap())?;
                        output.write_all(&buf[..])?;
                    } else {
                        let item = value.first().unwrap();

                        let buf: Vec<u8> = Vec::with_capacity(1000);
                        use bytes::BufMut;
                        let mut buf = buf.writer();

                        {
                            let mut sub_output = protobuf::CodedOutputStream::new(&mut buf);
                            other_message.write_bytes_from_fields(&mut sub_output, item)?;
                        }

                        let buf = buf.into_inner();
                        output.write_raw_varint64(u64::try_from(buf.len()).unwrap())?;
                        output.write_all(&buf[..])?;
                    }
                }
                _ => continue,
            }
        }
        Ok(())
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
    ) -> ServiceResult<Vec<Fields>> {
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
        let mut output: Vec<Fields> = Vec::with_capacity(2);
        while input.pos() < repeated_len && !input.eof()? {
            let len = input.pos() + input.read_raw_varint64()?;
            output.push(message.fields_from_bytes_delimited(input, len)?);
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

impl Serialize for ProtoEnum {
    fn serialize<S>(&self, sr: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Val(v) => sr.serialize_i32(*v),
        }
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

    use bytes::BufMut;

    use super::ServiceResult;

    #[test]
    fn test_bytes_from_fields() -> ServiceResult<()> {
        use super::*;
        use protobuf::descriptor::field_descriptor_proto::Label::{self, *};
        use protobuf::descriptor::field_descriptor_proto::Type::{self, *};

        let message2_fields: Fields = Fields::new(DashMap::new());
        message2_fields
            .map
            .insert("varint".to_string(), Some(Value::from_int32(150)));

        struct Table {
            name: String,
            field_type: Type,
            number: i32,
            type_name: String,
            label: Label,
            value: Value,
        }
        let table = [
            Table {
                name: "varint".to_string(),
                field_type: TYPE_INT32,
                number: 1,
                type_name: "".to_string(),
                label: LABEL_OPTIONAL,
                value: Value::from_int32(150),
            },
            Table {
                name: "string".to_string(),
                field_type: TYPE_STRING,
                number: 2,
                type_name: "".to_string(),
                label: LABEL_OPTIONAL,
                value: Value::from_string("testing".to_string()),
            },
            Table {
                name: "message".to_string(),
                field_type: TYPE_MESSAGE,
                number: 3,
                type_name: "Message2".to_string(),
                label: LABEL_OPTIONAL,
                value: Value::from_message(message2_fields.clone()),
            },
            Table {
                name: "message_repeated".to_string(),
                field_type: TYPE_MESSAGE,
                number: 4,
                type_name: "Message2".to_string(),
                label: LABEL_REPEATED,
                value: Value::Message(vec![message2_fields.clone(), message2_fields.clone()]),
            },
        ];

        let mut desc = protobuf::descriptor::DescriptorProto::new();
        let map = FieldsMap::new();
        for item in &table {
            let mut field = protobuf::descriptor::FieldDescriptorProto::new();
            field.set_name(item.name.clone());
            field.set_field_type(item.field_type);
            field.set_number(item.number);
            field.set_type_name(item.type_name.clone());
            field.set_label(item.label);
            desc.field.push(field);
            map.insert(item.name.clone(), Some(item.value.clone()));
        }

        let mut message2 = protobuf::descriptor::DescriptorProto::new();
        message2.field.push(desc.field.first().unwrap().clone());

        let parent: Arc<DashMap<String, Message>> = Arc::new(DashMap::new());
        parent.insert(
            "Message2".to_string(),
            Message::new(message2, "".to_string(), parent.clone()),
        );

        println!("Testing fieldstobytes");

        let m = Message::new(desc, "".to_string(), parent);
        let buf: Vec<u8> = Vec::with_capacity(1000);
        use bytes::BufMut;
        let mut buf = buf.writer();
        {
            let mut output = protobuf::CodedOutputStream::new(&mut buf);
            let fields = Fields::new(map);
            m.write_bytes_from_fields(&mut output, &fields)?;
        }

        let buf = buf.into_inner();
        let got = m.fields_from_bytes(&buf[..])?;
        for item in &table {
            let got_item = got.map.get(&item.name).unwrap().clone().unwrap();
            assert_eq!(item.value, got_item);
        }
        // let buf = buf.clone();
        // let buf = buf.as_ref();
        // buf.eq(want);
        // println!("{:?}", buf);

        Ok(())
    }

    #[test]
    fn test_fields_from_bytes() -> ServiceResult<()> {
        use super::*;
        use protobuf::descriptor::field_descriptor_proto::Label::{self, *};
        use protobuf::descriptor::field_descriptor_proto::Type::{self, *};

        let message2_fields: Fields = Fields::new(DashMap::new());
        message2_fields
            .map
            .insert("varint".to_string(), Some(Value::from_int32(150)));
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
                want: Value::from_message(message2_fields.clone()),
            },
            Table {
                name: "message_repeated".to_string(),
                field_type: TYPE_MESSAGE,
                number: 4,
                type_name: "Message2".to_string(),
                label: LABEL_REPEATED,
                want: Value::Message(vec![message2_fields.clone(), message2_fields.clone()]),
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
                .map
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
            fields_by_name: self.fields_by_name.clone(),
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

impl Default for Service {
    fn default() -> Self {
        Self {
            name: Default::default(),
            protocol: Protocol::None,
            methods: Default::default(),
            messages: Default::default(),
        }
    }
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
                let input_message = method.get_input_type().to_string();
                (
                    method.get_name().to_string(),
                    Method {
                        input_message: input_message.clone(),
                        output_message: method.get_output_type().to_string(),
                        handler: self.handler_from_http_api(&input_message, api.clone()),
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
        input_message: &String,
        api: http::API,
    ) -> Pin<Box<dyn Handler + Sync + Send + 'static>> {
        use crate::proto::http as proto_http;
        use ::http;
        fn method(pattern: proto_http::api::Pattern) -> http::Method {
            match pattern {
                proto_http::api::Pattern::get(_) => http::Method::GET,
                proto_http::api::Pattern::put(_) => http::Method::PUT,
                proto_http::api::Pattern::post(_) => http::Method::POST,
                proto_http::api::Pattern::delete(_) => http::Method::DELETE,
                proto_http::api::Pattern::patch(_) => http::Method::PATCH,
            }
        }
        let message = self.messages.get(input_message).unwrap();
        // let req_parts = http::request::Parts {
        //     method: method(api.pattern.unwrap()),
        //     uri: todo!(),
        //     version: todo!(),
        //     headers: todo!(),
        //     extensions: todo!(),
        // };
        // let resp_parts = http::response::Parts {
        //     status: todo!(),
        //     version: todo!(),
        //     headers: todo!(),
        //     extensions: todo!(),
        // };
        let writer = Http2Writer {};
        match api.content_type.as_str() {
            "application/json" => Box::pin(HttpJsonHandler::new(
                api.pattern.unwrap(),
                message.path.clone(),
                todo!(),
                todo!(),
                Box::new(writer),
            )),
            e => panic!("unknown http api content type: {}", e),
        }
    }

    pub async fn send_proto_to_local(&self, method: &String, data: &[u8]) -> ServiceResult<()> {
        let mut method = self.methods.get_mut(method).unwrap();
        let messages = self.messages.clone();
        let message = messages.get(&method.input_message).unwrap();
        let fields = Arc::new(message.fields_from_bytes(data)?);
        // let payload = method.handler.to_payload_and_send(&fields).await?;
        // let resp_fields = method.handler.from_payload(payload)?;
        // TODO: Proto from fields.
        Ok(())
    }
}

pub struct HttpJsonHandler {
    pub method: http::api::Pattern,
    prog: JSONQuery,
    path: String,
    req_parts: Arc<::http::request::Parts>,
    resp_parts: Arc<::http::response::Parts>,
    writer: Box<dyn Writer<Request = ::http::request::Parts, Response = ::http::response::Parts>>,
}

impl HttpJsonHandler {
    pub fn new(
        method: http::api::Pattern,
        path: String,
        req_parts: ::http::request::Parts,
        resp_parts: ::http::response::Parts,
        writer: Box<
            dyn Writer<Request = ::http::request::Parts, Response = ::http::response::Parts>,
        >,
    ) -> Self {
        Self {
            method,
            prog: JSONQuery::parse(path.as_str()).unwrap(),
            path,
            req_parts: Arc::new(req_parts),
            resp_parts: Arc::new(resp_parts),
            writer,
        }
    }
}

#[async_trait]
impl Handler for HttpJsonHandler {
    fn from_payload(&self, buf: bytes::Bytes) -> ServiceResult<Fields> {
        use bytes::Buf;
        let json: serde_json::Value = serde_json::from_reader(buf.reader()).unwrap();
        let pr = self.prog.execute(&json).unwrap();
        let result = pr.unwrap();
        Ok(serde_json::value::from_value(result)?)
    }

    async fn to_payload_and_send(&mut self, fields: &Fields) -> ServiceResult<bytes::Bytes> {
        match serde_json::to_vec(fields) {
            Ok(data) => {
                let data = bytes::Bytes::from_iter(data);
                todo!()
                // self.writer
                //     .write_request(self.req_parts.clone(), data)
                //     .await
            }
            Err(e) => Err(ServiceError::new(
                format!("to_payload json failed: {}", e.to_string()).as_str(),
            )),
        }
    }
}

#[async_trait]
pub trait Writer: Sync + Send {
    type Request;

    type Response;

    async fn write_request(
        self,
        context: Arc<::http::request::Parts>,
        send: h2::client::SendRequest<bytes::Bytes>,
        body: bytes::Bytes,
    ) -> ServiceResult<bytes::Bytes>;

    async fn write_response(
        self,
        context: Arc<Self::Response>,
        mut resp: h2::server::SendResponse<bytes::Bytes>,
        body: bytes::Bytes,
    ) -> ServiceResult<()>;
}

struct Http2Writer {}

#[async_trait]
impl Writer for Http2Writer {
    type Request = ::http::request::Parts;

    type Response = ::http::response::Parts;

    async fn write_request(
        self,
        context: Arc<::http::request::Parts>,
        send: h2::client::SendRequest<bytes::Bytes>,
        body: bytes::Bytes,
    ) -> ServiceResult<bytes::Bytes> {
        use ::http;
        let request = http::Request::from_parts(Arc::try_unwrap(context).unwrap(), ());
        let mut resp = send
            .ready()
            .await
            .and_then(|mut send_req| {
                let (resp, mut sender) = send_req.send_request(request, false)?;
                sender.send_data(body, true)?;
                Ok(resp)
            })
            .unwrap()
            .await
            .unwrap();
        let body = resp.body_mut();
        let body = match body.data().await {
            Some(body) => body,
            None => return Err(ServiceError::new("no body in response")),
        };
        match body {
            Ok(body) => Ok(body),
            Err(e) => Err(ServiceError::new(
                format!("error parsing body: {}", e).as_str(),
            )),
        }
    }

    async fn write_response(
        self,
        context: Arc<Self::Response>,
        mut resp: h2::server::SendResponse<bytes::Bytes>,
        body: bytes::Bytes,
    ) -> ServiceResult<()> {
        use ::http;
        let response = http::Response::from_parts(Arc::try_unwrap(context).unwrap(), ());
        let success = resp.send_response(response, false)?.send_data(body, true);
        match success {
            Ok(_) => Ok(()),
            Err(e) => Err(ServiceError::new(
                format!("error parsing body: {}", e).as_str(),
            )),
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
