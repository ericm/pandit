use std::convert::TryInto;

use async_trait::async_trait;

use crate::services::{Writer, WriterContext};

use tokio_postgres::{self, Client, NoTls};

pub struct PostgresWriter {
    client: Client,
}

#[async_trait]
impl Writer for PostgresWriter {
    async fn write_request(
        &mut self,
        context: WriterContext,
        fields: &crate::services::Fields,
        handler: &std::sync::Arc<dyn crate::services::Handler + Send + Sync>,
    ) -> crate::services::ServiceResult<bytes::Bytes> {
        let query = String::from_utf8(handler.to_payload(fields).await?.to_vec())?;
        let rows = self.client.query(&query, &[]).await?;
    }
}
