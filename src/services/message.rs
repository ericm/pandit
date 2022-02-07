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