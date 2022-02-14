use std::{io::Write, sync::Arc};

use dashmap::{mapref::one::Ref, DashMap};
use protobuf::{descriptor::FieldDescriptorProto, CodedInputStream};

use crate::services::{
    value::{ProtoEnum, Value},
    ServiceError,
};

use super::{Fields, FieldsMap, ServiceResult};
use std::convert::TryFrom;

pub struct Message {
    pub path: String,
    pub parent: Arc<DashMap<String, Message>>,
    fields: DashMap<u32, FieldDescriptorProto>,
    fields_by_name: DashMap<String, FieldDescriptorProto>,
    message: protobuf::descriptor::DescriptorProto,
}

macro_rules! as_variant {
    ($value:expr, $variant:path) => {
        match $value {
            $variant(x) => Ok(x),
            _ => Err(protobuf::ProtobufError::MessageNotInitialized(
                "incorrect pandit value".to_string(),
            )),
        }
    };
}

impl Message {
    pub(crate) fn new(
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
        let mut input = CodedInputStream::from_bytes(buf);
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
                value: T,
                wire_type: protobuf::wire_format::WireType,
            ) -> protobuf::ProtobufResult<()> {
                let num = u32::try_from(field.get_number()).unwrap();
                println!("write_value {} {}", field.get_name(), num);
                output.write_tag(num, wire_type)?;
                write(output, value)
            }

            fn write_value_repeated<'b, T: Clone>(
                field: Ref<String, FieldDescriptorProto>,
                output: &mut protobuf::CodedOutputStream<'b>,
                write: fn(&mut protobuf::CodedOutputStream<'b>, T) -> protobuf::ProtobufResult<()>,
                value: Vec<Value>,
                wire_type: protobuf::wire_format::WireType,
                extract_value: fn(Value) -> protobuf::ProtobufResult<T>,
            ) -> protobuf::ProtobufResult<()> {
                let num = u32::try_from(field.get_number()).unwrap();
                println!("write_value {} {}", field.get_name(), num);
                output.write_tag(num, protobuf::wire_format::WireTypeLengthDelimited)?;
                output.write_raw_varint64(u64::try_from(value.len()).unwrap())?;
                for item in value {
                    write(output, extract_value(item)?)?;
                }
                Ok(())
            }

            println!("match {} {:?}", field.get_name(), value);
            match field.get_field_type() {
                TYPE_DOUBLE => match value {
                    Value::Float(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_double_no_tag,
                        v.to_f64(),
                        protobuf::wire_format::WireTypeFixed64,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_double_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeFixed64,
                        |val| Ok(as_variant!(val, Value::Float)?.to_f64()),
                    )?,
                    _ => continue,
                },
                TYPE_FLOAT => match value {
                    Value::Float(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_float_no_tag,
                        v.to_f32(),
                        protobuf::wire_format::WireTypeFixed64,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_float_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeFixed32,
                        |val| Ok(as_variant!(val, Value::Float)?.to_f32()),
                    )?,
                    _ => continue,
                },
                TYPE_INT64 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int64_no_tag,
                        v.to_i64(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int64_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i64()),
                    )?,
                    _ => continue,
                },
                TYPE_UINT64 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint64_no_tag,
                        v.to_u64(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint64_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_u64()),
                    )?,
                    _ => continue,
                },
                TYPE_INT32 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int32_no_tag,
                        v.to_i32(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_int32_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i32()),
                    )?,
                    _ => continue,
                },
                TYPE_FIXED64 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed64_no_tag,
                        v.to_u64(),
                        protobuf::wire_format::WireTypeFixed64,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed64_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeFixed64,
                        |val| Ok(as_variant!(val, Value::Int)?.to_u64()),
                    )?,
                    _ => continue,
                },
                TYPE_FIXED32 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed32_no_tag,
                        v.to_u32(),
                        protobuf::wire_format::WireTypeFixed32,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_fixed32_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeFixed32,
                        |val| Ok(as_variant!(val, Value::Int)?.to_u32()),
                    )?,
                    _ => continue,
                },
                TYPE_BOOL => match value {
                    Value::Bool(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_bool_no_tag,
                        v,
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_bool_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| as_variant!(val, Value::Bool),
                    )?,
                    _ => continue,
                },
                TYPE_STRING => match value {
                    Value::String(v) => write_value(
                        field,
                        output,
                        |output, s: String| {
                            protobuf::CodedOutputStream::write_string_no_tag(output, s.as_str())
                        },
                        v,
                        protobuf::wire_format::WireTypeLengthDelimited,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        |output, s: String| {
                            protobuf::CodedOutputStream::write_string_no_tag(output, s.as_str())
                        },
                        arr,
                        protobuf::wire_format::WireTypeLengthDelimited,
                        |val| as_variant!(val, Value::String),
                    )?,
                    _ => continue,
                },
                TYPE_BYTES => match value {
                    Value::Bytes(v) => write_value(
                        field,
                        output,
                        |output, s: Vec<u8>| {
                            protobuf::CodedOutputStream::write_bytes_no_tag(output, &s[..])
                        },
                        v,
                        protobuf::wire_format::WireTypeLengthDelimited,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        |output, s: Vec<u8>| {
                            protobuf::CodedOutputStream::write_bytes_no_tag(output, &s[..])
                        },
                        arr,
                        protobuf::wire_format::WireTypeLengthDelimited,
                        |val| as_variant!(val, Value::Bytes),
                    )?,
                    _ => continue,
                },
                TYPE_UINT32 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint32_no_tag,
                        v.to_u32(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_uint32_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_u32()),
                    )?,
                    _ => continue,
                },
                TYPE_SFIXED32 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed32_no_tag,
                        v.to_i32(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed32_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i32()),
                    )?,
                    _ => continue,
                },
                TYPE_SFIXED64 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed64_no_tag,
                        v.to_i64(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sfixed64_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i64()),
                    )?,
                    _ => continue,
                },
                TYPE_SINT32 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint32_no_tag,
                        v.to_i32(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint32_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i32()),
                    )?,
                    _ => continue,
                },
                TYPE_SINT64 => match value {
                    Value::Int(v) => write_value(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint64_no_tag,
                        v.to_i64(),
                        protobuf::wire_format::WireTypeVarint,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        protobuf::CodedOutputStream::write_sint64_no_tag,
                        arr,
                        protobuf::wire_format::WireTypeVarint,
                        |val| Ok(as_variant!(val, Value::Int)?.to_i64()),
                    )?,
                    _ => continue,
                },
                TYPE_ENUM => match value {
                    Value::Enum(v) => write_value(
                        field,
                        output,
                        |output, val: ProtoEnum| {
                            use protobuf::ProtobufEnum;
                            protobuf::CodedOutputStream::write_enum_no_tag(output, val.value())
                        },
                        v,
                        protobuf::wire_format::WireTypeLengthDelimited,
                    )?,
                    Value::Array(arr) => write_value_repeated(
                        field,
                        output,
                        |output, val: ProtoEnum| {
                            use protobuf::ProtobufEnum;
                            protobuf::CodedOutputStream::write_enum_no_tag(output, val.value())
                        },
                        arr,
                        protobuf::wire_format::WireTypeLengthDelimited,
                        |val| as_variant!(val, Value::Enum),
                    )?,
                    _ => continue,
                },
                TYPE_MESSAGE => {
                    let message_name = field.get_type_name().to_string();
                    let parent = self.parent.clone();
                    let other_message = parent.get(&message_name).unwrap();
                    let num = u32::try_from(field.get_number()).unwrap();
                    output.write_tag(num, protobuf::wire_format::WireTypeLengthDelimited)?;
                    match value {
                        Value::Message(item) => {
                            let buf: Vec<u8> = Vec::with_capacity(1000);
                            use bytes::BufMut;
                            let mut buf = buf.writer();

                            {
                                let mut sub_output = protobuf::CodedOutputStream::new(&mut buf);
                                other_message.write_bytes_from_fields(&mut sub_output, &item)?;
                            }

                            let buf = buf.into_inner();
                            output.write_raw_varint64(u64::try_from(buf.len()).unwrap())?;
                            output.write_all(&buf[..])?;
                        }
                        Value::Array(arr) => {
                            let buf: Vec<u8> = Vec::with_capacity(10000);
                            use bytes::BufMut;
                            let mut buf = buf.writer();
                            {
                                let mut outer_output = protobuf::CodedOutputStream::new(&mut buf);
                                for item in arr {
                                    let item = as_variant!(item, Value::Message)?;
                                    let buf: Vec<u8> = Vec::with_capacity(1000);
                                    let mut buf = buf.writer();

                                    {
                                        let mut sub_output =
                                            protobuf::CodedOutputStream::new(&mut buf);
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
                        }
                        _ => continue,
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
        macro_rules! parse_vec {
            ($value:expr, $label:expr, $variant:path) => {
                if $label == Label::LABEL_REPEATED {
                    Some(Value::Array(
                        $value.iter().map(|v| $variant(v.clone())).collect(),
                    ))
                } else {
                    match $value.first() {
                        Some(v) => Some($variant(v.clone())),
                        None => Some(Value::None),
                    }
                }
            };
        }
        use protobuf::descriptor::field_descriptor_proto::Label;
        Ok((
            field.get_name().to_string(),
            match field.get_field_type() {
                TYPE_DOUBLE => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_double_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_float)
                }
                TYPE_FLOAT => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_float_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_float)
                }
                TYPE_INT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_int64_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_UINT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_uint64_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_INT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_int32_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_FIXED64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_fixed64_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_FIXED32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_fixed32_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_BOOL => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_bool_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::Bool)
                }
                TYPE_STRING => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_string_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::String)
                }
                TYPE_BYTES => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_bytes_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::Bytes)
                }
                TYPE_UINT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_uint32_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_SFIXED32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sfixed32_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_SFIXED64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sfixed64_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_SINT32 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sint32_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_SINT64 => {
                    let mut target = Vec::new();
                    protobuf::rt::read_repeated_sint64_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::from_int)
                }
                TYPE_ENUM => {
                    let mut target: Vec<ProtoEnum> = Vec::new();
                    protobuf::rt::read_repeated_enum_into(wire_type, input, &mut target)?;
                    parse_vec!(target, field.get_label(), Value::Enum)
                }
                TYPE_MESSAGE => {
                    let message_name = field.get_type_name().to_string();
                    let label = field.get_label();
                    match self.parse_another_message(input, &message_name, field) {
                        Ok(v) => parse_vec!(v, label, Value::Message),
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
            .insert("varint".to_string(), Some(Value::from_int(150)));

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
                value: Value::from_int(150),
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
                value: Value::Array(vec![
                    Value::Message(message2_fields.clone()),
                    Value::Message(message2_fields.clone()),
                ]),
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
            .insert("varint".to_string(), Some(Value::from_int(150)));
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
                want: Value::from_int(150),
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
                want: Value::Array(vec![
                    Value::Message(message2_fields.clone()),
                    Value::Message(message2_fields.clone()),
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
