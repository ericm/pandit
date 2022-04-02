use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{
    proto::gen::format::postgres::{
        exts::{postgres, postgres_field},
        Postgres, PostgresCommand, PostgresCondition,
    },
    services::{Fields, Method, ServiceResult},
};
use async_recursion::async_recursion;
use async_trait::async_trait;
use dashmap::{mapref::one::Ref, DashMap};
use postgres_types::{FromSql, IsNull, ToSql, Type};
use protobuf::descriptor::MethodOptions;
use sea_query::{
    tests_cfg::Char, ColumnDef, Expr, ForeignKey, ForeignKeyAction, Iden, IntoValueTuple, Query,
    SimpleExpr, Table, TableCreateStatement,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::{self, Client};

use crate::{
    proto,
    services::{
        message::{Field, Message},
        value::Value,
        FieldsMap, Handler, ServiceError,
    },
};

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
                let mut out = Vec::with_capacity(vals.len());
                for v in vals {
                    out.push(v.into_value());
                }
                sea_query::Value::Array(Some(Box::new(out)))
            }
            Value::Message(v) => sea_query::Value::Int(None),
            Value::None => sea_query::Value::Int(None),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SQLValue(pub Vec<u8>);

impl<'a> FromSql<'a> for SQLValue {
    fn from_sql(
        _: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Self(raw.to_vec()))
    }

    fn accepts(_: &postgres_types::Type) -> bool {
        true
    }
}

pub struct SQLHandler {
    messages: Arc<DashMap<String, Message>>,
    input_message: String,
    output_message: String,
    opts: Postgres,
}

impl SQLHandler {
    pub fn new(
        messages: Arc<DashMap<String, Message>>,
        input_message: String,
        output_message: String,
        opts: Postgres,
    ) -> Self {
        Self {
            messages,
            input_message,
            output_message,
            opts,
        }
    }
}

macro_rules! handle_err {
    ($value:expr) => {
        match $value {
            Ok(v) => v,
            Err(e) => {
                return Err(ServiceError::new(
                    format!("conversion error: {:?}", e).as_str(),
                ));
            }
        }
    };
}

#[async_trait]
impl Handler for SQLHandler {
    fn from_payload(&self, buf: bytes::Bytes) -> ServiceResult<Fields> {
        use protobuf::descriptor::field_descriptor_proto::Type::*;
        let buf = &buf.to_vec()[..];
        let rows: Vec<HashMap<String, SQLValue>> = serde_json::from_slice(buf)?;
        for map in rows {
            let fields = FieldsMap::default();
            let message = {
                self.messages
                    .get(&self.output_message)
                    .ok_or("output message not found")?
            };
            for (name, value) in map {
                let value = match message
                    .fields_by_name
                    .get(&name)
                    .ok_or("no field")?
                    .descriptor
                    .get_field_type()
                {
                    TYPE_DOUBLE => {
                        Value::from_float(handle_err!(<f64>::from_sql(&Type::FLOAT8, &value.0[..])))
                    }
                    TYPE_FLOAT => {
                        Value::from_float(handle_err!(<f32>::from_sql(&Type::FLOAT4, &value.0[..])))
                    }
                    TYPE_BOOL => {
                        Value::Bool(handle_err!(<bool>::from_sql(&Type::BOOL, &value.0[..])))
                    }
                    TYPE_STRING => Value::from_string(handle_err!(<String>::from_sql(
                        &Type::TEXT,
                        &value.0[..]
                    ))),
                    TYPE_BYTES => {
                        Value::Bytes(handle_err!(<Vec<u8>>::from_sql(&Type::BYTEA, &value.0[..])))
                    }
                    TYPE_MESSAGE => {
                        // TODO: populate sub_table from message.parents and create foreign key.
                        let table_name = message
                            .fields_by_name
                            .get(&name)
                            .ok_or("no field")?
                            .descriptor
                            .get_type_name();
                        todo!()
                    }
                    TYPE_INT64 | TYPE_UINT64 | TYPE_INT32 | TYPE_UINT32 | TYPE_FIXED64
                    | TYPE_FIXED32 | TYPE_SFIXED32 | TYPE_SFIXED64 | TYPE_SINT32 | TYPE_SINT64
                    | TYPE_ENUM => {
                        Value::from_int(handle_err!(<i64>::from_sql(&Type::INT8, &value.0[..])))
                    }
                    _ => return Err(ServiceError::new("unsupported proto field type")),
                };
                fields.insert(name, Some(value));
            }
        }
        Ok(Fields::new(fields))
    }

    async fn to_payload(&self, fields: &Fields) -> ServiceResult<bytes::Bytes> {
        let mut cmds = HashMap::<String, String>::with_capacity(1);
        let message = {
            self.messages
                .get(&self.input_message)
                .ok_or("no input message")?
        };
        self._to_payload(message, &mut cmds, fields)?;
        Ok(bytes::Bytes::from(serde_json::to_string(&cmds)?))
    }
}

impl SQLHandler {
    fn _to_payload(
        &self,
        message: Ref<String, Message>,
        cmds: &mut HashMap<String, String>,
        fields: &Fields,
    ) -> ServiceResult<sea_query::Value> {
        let mut vals = Vec::with_capacity(fields.map.len());
        let mut cols = Vec::<Field>::with_capacity(fields.map.len());
        let mut primary_key: Option<sea_query::Value> = None;
        for entry in fields.map.iter() {
            vals.push(match entry.value() {
                Some(value) => match value {
                    Value::Message(other_fields) => {
                        let field = {
                            let m = message.fields_by_name.get(entry.key());
                            let m = m.ok_or("no field error")?;
                            m.value().clone()
                        };
                        let message_name = field.descriptor.get_type_name().to_string();
                        let other_message =
                            self.messages.get(&message_name).ok_or("no message found")?;
                        self._to_payload(other_message, cmds, other_fields)?
                    }
                    _ => value.clone().into_value(),
                },
                None => sea_query::Value::Int(None),
            });
            let col = {
                let m = message.fields_by_name.get(entry.key());
                let m = m.ok_or("no field error")?;
                m.value().clone()
            };
            if match postgres_field.get(col.descriptor.options.as_ref().ok_or("no field options")?)
            {
                Some(field_opts) => field_opts.key,
                None => false,
            } {
                primary_key = Some(vals.last().unwrap().clone());
            }
            cols.push(col.clone());
        }
        let primary_key = primary_key.ok_or("no primary key")?;
        match self.opts.command.enum_value().unwrap_or_default() {
            PostgresCommand::INSERT => {
                let mut query = Query::insert();
                let query = query
                    .into_table(message.value().clone())
                    .columns(cols)
                    .values(vals)?;
                cmds.insert(
                    message.key().clone(),
                    query.to_string(sea_query::PostgresQueryBuilder),
                );
            }
            PostgresCommand::DELETE => {
                let mut query = Query::delete();
                for (val, col) in vals.iter().zip(cols.iter()) {
                    match val {
                        sea_query::Value::Int(v) => match v {
                            Some(_) => {}
                            None => continue,
                        },
                        _ => {}
                    }
                    let cond = match postgres_field
                        .get(col.descriptor.options.as_ref().unwrap_or_default())
                    {
                        Some(opts) => match opts.condition.enum_value() {
                            Ok(opts) => opts,
                            Err(_) => continue,
                        },
                        None => continue,
                    };
                    match cond {
                        PostgresCondition::EQ => {
                            query.and_where(Expr::col(col.clone()).eq(val.clone()));
                        }
                        PostgresCondition::NE => {
                            query.and_where(Expr::col(col.clone()).ne(val.clone()));
                        }
                        PostgresCondition::LE => {
                            query.and_where(
                                Expr::col(col.clone()).less_or_equal(Expr::val(val.clone())),
                            );
                        }
                        PostgresCondition::LT => {
                            query.and_where(
                                Expr::col(col.clone()).less_than(Expr::val(val.clone())),
                            );
                        }
                        PostgresCondition::GE => {
                            query.and_where(
                                Expr::col(col.clone()).greater_or_equal(Expr::val(val.clone())),
                            );
                        }
                        PostgresCondition::GT => {
                            query.and_where(
                                Expr::col(col.clone()).greater_than(Expr::val(val.clone())),
                            );
                        }
                    };
                }
                cmds.insert(
                    message.key().clone(),
                    query.to_string(sea_query::PostgresQueryBuilder),
                );
            }
            PostgresCommand::UPDATE => todo!(), // TODO: Implement
            PostgresCommand::SELECT => todo!(), // TODO: Implement
        };
        Ok(primary_key)
    }
}
