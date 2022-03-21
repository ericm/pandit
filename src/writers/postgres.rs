pub struct PostgresWriter {}

use async_trait::async_trait;
use diesel;

use crate::services::{Writer, WriterContext};

#[async_trait]
impl Writer for PostgresWriter {
    async fn write_request(
        &mut self,
        context: WriterContext,
        fields: &crate::services::Fields,
        handler: &std::sync::Arc<dyn crate::services::Handler + Send + Sync>,
    ) -> crate::services::ServiceResult<bytes::Bytes> {
        // diesel::insert_into(target)
        // diesel::dyn
    }
}
