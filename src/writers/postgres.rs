use std::{collections::HashMap, convert::TryInto};

use async_trait::async_trait;

use crate::{
    handlers::sql::SQLValue,
    services::{Writer, WriterContext},
};
use postgres_types::{FromSql, IsNull, ToSql};

use serde_json;
use tokio_postgres::{self, Client, NoTls};

pub struct PostgresWriter {
    client: Client,
}

#[async_trait]
impl Writer for PostgresWriter {
    async fn write_request(
        &mut self,
        _: WriterContext,
        fields: &crate::services::Fields,
        handler: &std::sync::Arc<dyn crate::services::Handler + Send + Sync>,
    ) -> crate::services::ServiceResult<bytes::Bytes> {
        let query = String::from_utf8(handler.to_payload(fields).await?.to_vec())?;
        let rows = self.client.query_opt(&query, &[]).await?;
        let row = rows.ok_or("no rows in response")?;

        let mut out = HashMap::<String, SQLValue>::with_capacity(row.len());
        let cols = row.columns();
        for i in 0..row.len() {
            out.insert(cols[i].name().to_string(), row.get(i));
        }
        Ok(bytes::Bytes::copy_from_slice(
            &serde_json::to_vec(&out)?[..],
        ))
    }
}
