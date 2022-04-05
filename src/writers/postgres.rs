use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    net::SocketAddr,
};

use async_trait::async_trait;

use crate::{
    handlers::sql::SQLValue,
    services::{Fields, Handler, ServiceResult, Writer, WriterContext},
};
use postgres_types::{FromSql, IsNull, ToSql};

use serde_json;
use tokio_postgres::{self, Client, NoTls};

use super::LoadBalancer;

pub struct PostgresWriter {
    lb: LoadBalancer,
}

impl PostgresWriter {
    pub fn new(lb: LoadBalancer) -> ServiceResult<Self> {
        Ok(Self { lb })
    }
}

#[async_trait]
impl Writer for PostgresWriter {
    async fn write_request(
        &mut self,
        _: WriterContext,
        fields: &Fields,
        handler: &std::sync::Arc<dyn Handler + Send + Sync>,
    ) -> ServiceResult<bytes::Bytes> {
        let addr = self.lb.get_addr().await;
        let addr: SocketAddr = addr.parse()?;
        // Authentication configuration not currently supported.
        let config = format!(
            "host={} port={} user=root dbname=root",
            addr.ip(),
            addr.port()
        );
        let (client, conn) = tokio_postgres::connect(&config, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                log::error!("connection error: {}", e);
            }
        });
        let queries = handler.to_payload(fields).await?;
        use bytes::Buf;
        let queries: Vec<(String, String)> = serde_json::from_reader(queries.reader())?;
        let mut out_rows = Vec::<(String, HashMap<String, SQLValue>)>::with_capacity(queries.len());
        for (name, query) in queries {
            log::info!("pg: executing query '{}'", query);
            let rows = client.query_opt(&query, &[]).await?;
            let row = match rows {
                Some(v) => v,
                None => continue,
            };

            let mut out = HashMap::<String, SQLValue>::with_capacity(row.len());
            let cols = row.columns();
            for i in 0..row.len() {
                out.insert(cols[i].name().to_string(), row.get(i));
            }
            out_rows.push((name, out));
        }

        Ok(bytes::Bytes::copy_from_slice(
            &serde_json::to_vec(&out_rows)?[..],
        ))
    }
}
