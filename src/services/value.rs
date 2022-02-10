use protobuf::reflect::{runtime_types::RuntimeTypeEnum, ProtobufValue};
use serde::{de::Visitor, Deserialize, Serialize};

use super::{Fields, FieldsMap};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Bytes(Vec<u8>),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float64(f64),
    Float32(f32),
    Bool(bool),
    Enum(ProtoEnum),
    Message(Fields),
    Array(Vec<Value>),
    None,
}

impl Value {
    pub fn from_string(val: String) -> Self {
        Self::String(val)
    }

    pub fn from_int32(val: i32) -> Self {
        Self::Int32(val)
    }

    pub fn from_uint32(val: u32) -> Self {
        Self::UInt32(val)
    }

    pub fn from_message(fields: Fields) -> Self {
        Self::Message(fields)
    }
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
        use protobuf::ProtobufEnum;
        match self {
            Value::String(v) => sr.serialize_str(v.as_str()),
            Value::Bytes(v) => sr.serialize_bytes(&v[..]),
            Value::Int32(v) => sr.serialize_i32(*v),
            Value::Int64(v) => sr.serialize_i64(*v),
            Value::UInt32(v) => sr.serialize_u32(*v),
            Value::UInt64(v) => sr.serialize_u64(*v),
            Value::Float64(v) => sr.serialize_f64(*v),
            Value::Float32(v) => sr.serialize_f32(*v),
            Value::Bool(v) => sr.serialize_bool(*v),
            Value::Enum(v) => sr.serialize_i32(v.value()),
            Value::Message(v) => v.serialize(sr),
            Value::Array(v) => serialize_seq(sr, &v),
            Value::None => sr.serialize_none(),
        }
    }
}

struct ValueVisitor {}
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a pandit supported value")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut value = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(item) = seq.next_element()? {
            value.push(item);
        }
        Ok(Value::Array(value))
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_string(v))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(v.to_vec()))
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Int32(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Int64(v))
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::UInt32(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::UInt64(v))
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float32(v))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float64(v))
    }
    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let map = FieldsMap::new();
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }
        Ok(Value::Message(Fields::new(map)))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::None)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(dr: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = ValueVisitor {};
        dr.deserialize_any(visitor)
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
                l0.map.iter().all(|k| r0.map.contains_key(k.key()))
            }
            (Self::Array(l0), Self::Array(r0)) => {
                l0.len() == r0.len() && l0.iter().zip(r0).all(|(l, r)| l == r)
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
