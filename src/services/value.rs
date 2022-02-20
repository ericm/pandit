use std::{
    cmp::Ordering,
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{Hash, Hasher},
    io::ErrorKind,
    str::Utf8Error,
    sync::Arc,
};

use protobuf::reflect::{runtime_types::RuntimeTypeEnum, ProtobufValue};
use redis::{FromRedisValue, ToRedisArgs};
use serde::{de::Visitor, Deserialize, Serialize};

use super::{Fields, FieldsMap};

pub trait Integer: Sync + Send + std::fmt::Debug {
    fn to_i32(&self) -> i32;
    fn to_i64(&self) -> i64;
    fn to_u32(&self) -> u32;
    fn to_u64(&self) -> u64;
}

impl Integer for i32 {
    fn to_i32(&self) -> i32 {
        self.clone()
    }

    fn to_i64(&self) -> i64 {
        self.clone().into()
    }

    fn to_u32(&self) -> u32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_u64(&self) -> u64 {
        self.clone().try_into().unwrap_or(0)
    }
}

impl Integer for i64 {
    fn to_i32(&self) -> i32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_i64(&self) -> i64 {
        self.clone()
    }

    fn to_u32(&self) -> u32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_u64(&self) -> u64 {
        self.clone().try_into().unwrap_or(0)
    }
}

impl Integer for u32 {
    fn to_i32(&self) -> i32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_i64(&self) -> i64 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_u32(&self) -> u32 {
        self.clone()
    }

    fn to_u64(&self) -> u64 {
        self.clone().into()
    }
}

impl Integer for u64 {
    fn to_i32(&self) -> i32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_i64(&self) -> i64 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_u32(&self) -> u32 {
        self.clone().try_into().unwrap_or(0)
    }

    fn to_u64(&self) -> u64 {
        self.clone()
    }
}

impl Hash for dyn Integer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_i64(self.to_i64())
    }
}

impl PartialEq for dyn Integer {
    fn eq(&self, other: &Self) -> bool {
        self.to_i64() == other.to_i64()
    }
}

impl Eq for dyn Integer {}

pub trait Floating: Sync + Send + std::fmt::Debug {
    fn to_f32(&self) -> f32;
    fn to_f64(&self) -> f64;
}

impl Floating for f32 {
    fn to_f32(&self) -> f32 {
        self.clone()
    }

    fn to_f64(&self) -> f64 {
        self.clone().into()
    }
}

impl Floating for f64 {
    fn to_f32(&self) -> f32 {
        num::ToPrimitive::to_f32(self).unwrap_or(0f32)
    }

    fn to_f64(&self) -> f64 {
        self.clone()
    }
}

impl Hash for dyn Floating {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.to_f64().to_be_bytes())
    }
}

impl PartialEq for dyn Floating {
    fn eq(&self, other: &Self) -> bool {
        self.to_f64().to_bits() == other.to_f64().to_bits()
    }
}

impl Eq for dyn Floating {}

#[derive(Debug, Clone, Hash, Eq)]
pub enum Value {
    String(String),
    Bytes(Vec<u8>),
    Int(Arc<dyn Integer>),
    Float(Arc<dyn Floating>),
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

    pub fn from_int<T>(val: T) -> Self
    where
        T: Integer + 'static,
    {
        Self::Int(Arc::new(val))
    }

    pub fn from_float<T>(val: T) -> Self
    where
        T: Floating + 'static,
    {
        Self::Float(Arc::new(val))
    }

    pub fn from_message(fields: Fields) -> Self {
        Self::Message(fields)
    }
}

impl ToRedisArgs for Value {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let buf = serde_json::to_vec(self).unwrap();
        out.write_arg(&buf[..]);
    }
}

impl FromRedisValue for Value {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Data(v) => {
                let result: serde_json::Value = serde_json::from_slice(&v[..]).unwrap();
                Ok(serde_json::value::from_value(result).unwrap())
            }
            _ => Err(redis::RedisError::from(std::io::Error::new(
                ErrorKind::Other,
                "byte vec required",
            ))),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut s_hash = DefaultHasher::new();
        self.hash(&mut s_hash);
        let mut o_hash = DefaultHasher::new();
        other.hash(&mut o_hash);
        let s_hash = s_hash.finish();
        let o_hash = o_hash.finish();
        if s_hash == o_hash {
            Some(Ordering::Equal)
        } else if s_hash > o_hash {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).unwrap();
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
            Value::Int(v) => sr.serialize_i64(v.to_i64()),
            Value::Float(v) => sr.serialize_f64(v.to_f64()),
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
        Ok(Value::from_int(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_int(v))
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_int(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_int(v))
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_float(v))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_float(v))
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
            (Self::Int(l0), Self::Int(r0)) => l0.to_i64() == r0.to_i64(),
            (Self::Float(l0), Self::Float(r0)) => l0.to_f64() == r0.to_f64(),
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
