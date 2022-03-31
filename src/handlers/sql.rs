use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{
    proto::gen::format::postgres::{
        exts::{postgres, postgres_field},
        Postgres, PostgresCommand,
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
        Fields, FieldsMap, Handler, ServiceError, ServiceResult,
    },
};
pub struct SQLHandler {
    messages: Arc<DashMap<String, Message>>,
    method: Method,
    opts: MethodOptions,
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
                let mut out = Vec::with_capacity(vals.len());
                for v in vals {
                    out.push(v.into_value());
                }
                sea_query::Value::Array(Some(Box::new(out)))
            }
            Value::Message(v) => {
                // let (other_message, other_primary_key) = {
                //     let other_message = message
                //         .parent
                //         .get(&field.descriptor.get_type_name().to_string())
                //         .unwrap();
                //     let other_message = other_message.value();
                //     let mut table = Table::create();
                //     let table = table.table(other_message.clone()).if_not_exists();
                //     populate_table(message, table, client).await;
                //     client
                //         .execute(&table.to_string(sea_query::PostgresQueryBuilder), &[])
                //         .await
                //         .unwrap();
                //     (
                //         other_message.clone(),
                //         primary_key_for_message(message).unwrap(),
                //     )
                // };
                // table.foreign_key(
                //     ForeignKey::create()
                //         .name(field.descriptor.get_name())
                //         .from(message.clone(), field.clone())
                //         .to(other_message, other_primary_key)
                //         .on_delete(ForeignKeyAction::Cascade)
                //         .on_update(ForeignKeyAction::Cascade),
                // );
            }
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

fn primary_key_for_message(message: &Message) -> Option<Field> {
    for entry in message.fields_by_name.iter() {
        let opts = entry.value().descriptor.options.get_ref();
        let is_key = proto::gen::pandit::exts::key.get(opts);
        if is_key.unwrap_or_default() {
            return Some(entry.value().clone());
        }
    }
    None
}

impl SQLHandler {
    pub fn new(
        messages: Arc<DashMap<String, Message>>,
        method: Method,
        opts: MethodOptions,
    ) -> Result<Self, Box<(dyn Error + 'static)>> {
        Ok(Self {
            messages,
            method,
            opts,
        })
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
        let map: HashMap<String, SQLValue> = serde_json::from_slice(buf)?;
        let fields = FieldsMap::default();
        for (name, value) in map {
            let value = match self
                .message
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
                TYPE_BOOL => Value::Bool(handle_err!(<bool>::from_sql(&Type::BOOL, &value.0[..]))),
                TYPE_STRING => {
                    Value::from_string(handle_err!(<String>::from_sql(&Type::TEXT, &value.0[..])))
                }
                TYPE_BYTES => {
                    Value::Bytes(handle_err!(<Vec<u8>>::from_sql(&Type::BYTEA, &value.0[..])))
                }
                TYPE_MESSAGE => {
                    // TODO: populate sub_table from message.parents and create foreign key.
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
        Ok(Fields::new(fields))
    }

    async fn to_payload(&self, fields: &Fields) -> ServiceResult<bytes::Bytes> {
        let mut cmds = Vec::<String>::with_capacity(1);
        let message = {
            self.messages
                .get(&self.method.input_message)
                .ok_or("no input message")?
        };
        self._to_payload(
            message,
            &mut cmds,
            fields,
            postgres
                .get(&self.opts)
                .as_ref()
                .ok_or("no postgres options")?,
        );
        Ok(bytes::Bytes::from(serde_json::to_string(&cmds)?))
    }
}

impl SQLHandler {
    fn _to_payload(
        &self,
        message: Ref<String, Message>,
        cmds: &mut Vec<String>,
        fields: &Fields,
        opts: &Postgres,
    ) -> ServiceResult<sea_query::Value> {
        let mut vals = Vec::with_capacity(fields.map.len());
        let mut cols = Vec::<Field>::with_capacity(fields.map.len());
        let mut primary_key: Option<Value> = None;
        for entry in fields.map.iter() {
            vals.push(match entry.value() {
                Some(value) => match value {
                    Value::Message(other_fields) => {
                        let field = message
                            .fields_by_name
                            .get(entry.key())
                            .ok_or("no field error")?
                            .value();
                        let message_name = field.descriptor.get_type_name().to_string();
                        let other_message =
                            self.messages.get(&message_name).ok_or("no message found")?;
                        let foreign_key = self._to_payload(other_message, cmds, other_fields, opts)?;
                        value.clone().into_value()
                    },
                    _ => value.clone().into_value(),
                },
                None => sea_query::Value::Int(None),
            });
            let col = message
                .fields_by_name
                .get(entry.key())
                .ok_or("no field error")?
                .value();
            if match postgres_field.get(col.descriptor.options.as_ref().ok_or("no field options")?) {
                Some(field_opts) => field_opts.key,
                None => false,
            } {
                primary_key = Some(vals.last().unwrap());
            }
            cols.push(col.clone());
        }
        let primary_key = primary_key.ok_or("no primary key")?;
        match opts.command.enum_value().unwrap_or_default() {
            PostgresCommand::INSERT => {
                let mut query = Query::insert();
                let query = query
                    .into_table(message.value().clone())
                    .columns(cols)
                    .values(vals)?;
                cmds.push(query.to_string(sea_query::PostgresQueryBuilder));
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
                    query.and_where(Expr::col(col.clone()).eq(val.clone()));
                }
                cmds.push(query.to_string(sea_query::PostgresQueryBuilder));
            }
            PostgresCommand::UPDATE => {}
            PostgresCommand::SELECT => {}
        };
        Ok(primary_key)
    }
}
