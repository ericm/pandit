use std::path::PathBuf;

use crate::proto::gen::format;
use crate::services::{ServiceError, ServiceResult, WriterRef};

pub mod http;

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
    match format::http::exts::http_service.get(&service.options.as_ref().unwrap_or_default()) {
        Some(service) => match service.version.unwrap() {
            format::http::HTTPVersion::VERSION_1_0 => {}
            format::http::HTTPVersion::VERSION_1_1 => todo!(),
            format::http::HTTPVersion::VERSION_2_0 => todo!(),
        },
        None => {}
    };

    Err(ServiceError::new("no format defined in service"))
}
