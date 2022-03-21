use std::sync::Arc;

use async_trait::async_trait;
use postgres_types::{IsNull, ToSql};
use tokio_postgres;

use crate::services::{message::Message, value::Value, Fields, Handler, ServiceResult};

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        match &self {
            Value::String(s) => s.to_sql(ty, out),
            Value::Bytes(b) => b.to_sql(ty, out),
            Value::Int(i) => {
                let i = i.to_i64();
                i.to_sql(ty, out)
            }
            Value::Float(f) => {
                let f = f.to_f64();
                f.to_sql(ty, out)
            }
            Value::Bool(b) => b.to_sql(ty, out),
            Value::Enum(e) => {
                use protobuf::ProtobufEnum;
                let e = e.value();
                e.to_sql(ty, out)
            }
            Value::Array(a) => a.to_sql(ty, out),
            Value::Message(f) => {}
            Value::None => Ok(IsNull::Yes),
        }
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        todo!()
    }
}

// impl ToSql for Fields {
//     fn to_sql(
//         &self,
//         ty: &postgres_types::Type,
//         out: &mut bytes::BytesMut,
//     ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
//     where
//         Self: Sized,
//     {
//         let j = postgres_types::Json(self);
//         j.to_sql(ty, out)
//     }

//     fn accepts(ty: &postgres_types::Type) -> bool
//     where
//         Self: Sized,
//     {
//         true
//     }

//     fn to_sql_checked(
//         &self,
//         ty: &postgres_types::Type,
//         out: &mut bytes::BytesMut,
//     ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
//         todo!()
//     }
// }

pub struct SQLHandler {}

impl SQLHandler {
    pub fn new(table_name: &String, message: &Message) -> ServiceResult<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Handler for SQLHandler {
    fn from_payload(
        &self,
        buf: bytes::Bytes,
    ) -> crate::services::ServiceResult<crate::services::Fields> {
        todo!()
    }

    async fn to_payload(
        &self,
        fields: &crate::services::Fields,
    ) -> crate::services::ServiceResult<bytes::Bytes> {
        todo!()
    }
}
