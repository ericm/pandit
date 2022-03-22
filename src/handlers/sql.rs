use std::sync::Arc;

use async_trait::async_trait;
use postgres_types::{FromSql, IsNull, ToSql};
use sea_query::{
    tests_cfg::Char, ColumnDef, Iden, IntoValueTuple, Query, Table, TableCreateStatement,
};
use tokio_postgres;

use crate::services::{
    message::{Field, Message},
    value::Value,
    Fields, Handler, ServiceError, ServiceResult,
};
pub struct SQLHandler {
    message: Message,
}

impl Iden for Message {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.name).unwrap()
    }
}

impl Iden for Field {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.descriptor.get_name()).unwrap()
    }
}

impl Value {
    fn into_value(self) -> sea_query::Value {
        match self {
            Value::String(v) => sea_query::Value::String(Some(Box::new(v))),
            Value::Bytes(v) => sea_query::Value::Bytes(Some(Box::new(v))),
            Value::Int(v) => sea_query::Value::Int(Some(v.to_i32())),
            Value::Float(v) => sea_query::Value::Float(Some(v.to_f32())),
            Value::Bool(v) => sea_query::Value::Bool(Some(v)),
            Value::Enum(v) => sea_query::Value::Int(Some(protobuf::ProtobufEnum::value(&v))),
            Value::Array(vals) => {
                let out = Vec::with_capacity(vals.len());
                for v in vals {
                    out.push(v.into_value());
                }
                sea_query::Value::Array(Some(Box::new(out)))
            }
            Value::Message(v) => todo!(),
            Value::None => sea_query::Value::Int(None),
        }
    }
}

pub struct SQLValue(pub Vec<u8>);

impl<'a> FromSql<'a> for SQLValue {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Self(raw.to_vec()))
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        true
    }
}

// impl Iden for V

fn populate_table(message: &Message, table: &mut TableCreateStatement) -> ServiceResult<()> {
    use protobuf::descriptor::field_descriptor_proto::Label::*;
    use protobuf::descriptor::field_descriptor_proto::Type::*;

    for (_, field) in message.fields_by_name {
        let mut def = ColumnDef::new(field);
        match field.descriptor.get_label() {
            LABEL_OPTIONAL => {}
            LABEL_REQUIRED => {
                def.not_null();
            }
            LABEL_REPEATED => {
                return Err(ServiceError::new(
                    "repeated label is not currently supported in SQL",
                ))
            }
        };
        match field.descriptor.get_field_type() {
            TYPE_DOUBLE => def.double(),
            TYPE_FLOAT => def.float(),
            TYPE_BOOL => def.boolean(),
            TYPE_STRING => def.string(),
            TYPE_BYTES => def.binary(),
            TYPE_MESSAGE => {
                // TODO: populate sub_table from message.parents and create foreign key.
                todo!()
            }
            TYPE_INT64 | TYPE_UINT64 | TYPE_INT32 | TYPE_UINT32 | TYPE_FIXED64 | TYPE_FIXED32
            | TYPE_SFIXED32 | TYPE_SFIXED64 | TYPE_SINT32 | TYPE_SINT64 | TYPE_ENUM => {
                def.integer()
            }
            _ => &mut def,
        };
        table.col(&mut def);
    }
    Ok(())
}

impl SQLHandler {
    pub fn new(message: &Message) -> ServiceResult<Self> {
        let table = Table::create().table(message.clone()).if_not_exists();
        populate_table(message, table)?;
        Ok(Self {
            message: message.clone(),
        })
    }

    fn cols(&self) -> Vec<Field> {
        self.message
            .fields_by_name
            .iter()
            .map(|v| v.value().clone())
            .collect()
    }
}

#[async_trait]
impl Handler for SQLHandler {
    fn from_payload(
        &self,
        buf: bytes::Bytes,
    ) -> crate::services::ServiceResult<crate::services::Fields> {
        let buf = buf.to_vec();
        todo!()
    }

    async fn to_payload(
        &self,
        fields: &crate::services::Fields,
    ) -> crate::services::ServiceResult<bytes::Bytes> {
        let vals = Vec::with_capacity(fields.map.len());
        let cols = Vec::<Field>::with_capacity(fields.map.len());
        for (name, value) in fields.map {
            vals.push(match value {
                Some(value) => value.into_value(),
                None => sea_query::Value::Int(None),
            });
            cols.push(
                self.message
                    .fields_by_name
                    .get(&name)
                    .ok_or("no field error")?
                    .value()
                    .clone(),
            );
        }
        let query = Query::insert()
            .into_table(self.message)
            .columns(cols)
            .values(vals)?;
        Ok(bytes::Bytes::from(
            query.to_string(sea_query::PostgresQueryBuilder),
        ))
    }
}
