use std::convert::TryInto;

use async_trait::async_trait;

use crate::{
    handlers::sql::SQLValue,
    services::{Writer, WriterContext},
};
use postgres_types::{FromSql, IsNull, ToSql};

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

        let mut out = Vec::<SQLValue>::with_capacity(row.len());
        for i in 0..row.len() {
            out.push(row.get(i));
        }
        let out: Vec<Vec<u8>> = out.iter().map(|v| v.0.clone()).collect();
        let out = out.join(&[0xff, 0xff, 0xff, 0xff, 0xff][..]);
        Ok(bytes::Bytes::copy_from_slice(&out[..]))
    }
}
