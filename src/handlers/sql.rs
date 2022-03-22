use std::sync::Arc;

use async_trait::async_trait;
use postgres_types::{IsNull, ToSql};
use sea_query::{tests_cfg::Char, ColumnDef, Iden, Table, TableCreateStatement};
use tokio_postgres;

use crate::services::{
    message::{Field, Message},
    value::Value,
    Fields, Handler, ServiceError, ServiceResult,
};
pub struct SQLHandler {}

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

fn populate_table(message: &Message, table: &mut TableCreateStatement) -> ServiceResult<()> {
    use protobuf::descriptor::field_descriptor_proto::Label::*;
    use protobuf::descriptor::field_descriptor_proto::Type::*;

    for (name, field) in message.fields_by_name {
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
            TYPE_MESSAGE => todo!(),
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
