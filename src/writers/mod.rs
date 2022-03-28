use std::path::PathBuf;

use tokio::sync::Mutex;

use crate::proto::gen::format;
use crate::proto::gen::format::http::exts::http_service;
use crate::proto::gen::format::postgres::exts::postgres_service;
use crate::services::{ServiceError, ServiceResult, WriterRef};

use self::http::HttpWriter;

pub mod http;
pub mod postgres;

pub fn writer_from_proto(
    proto_path: PathBuf,
    includes: &[PathBuf],
    addr: &str,
) -> ServiceResult<WriterRef> {
    let parsed = protobuf_parse::pure::parse_and_typecheck(includes, &[proto_path.clone()])?;
    let filename = proto_path.file_name().unwrap().to_str().unwrap();
    let file = parsed
        .file_descriptors
        .iter()
        .find(|&x| {
            let name = x.get_name().to_string();
            name.ends_with(filename)
        })
        .unwrap();
    let service = file.service.first().unwrap();

    // Generate writer.
    let options = service.options.as_ref().unwrap_or_default();
    match http_service.get(&options) {
        Some(service) => {
            let version = service.version.unwrap();
            return Ok(Box::new(Mutex::new(HttpWriter::new(addr, version))));
        }
        None => {}
    };
    match postgres_service.get(&options) {
        Some(_) => return Ok(Box::new(Mutex::new(postgres::PostgresWriter::new(addr)?))),
        None => {}
    };

    Err(ServiceError::new("no format defined in service"))
}
