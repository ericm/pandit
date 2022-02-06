use protobuf::reflect::{runtime_types::RuntimeTypeEnum, ProtobufValue};
use serde::{de::Visitor, Deserialize, Serialize};

use super::Fields;

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
