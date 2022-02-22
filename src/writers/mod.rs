use std::path::PathBuf;

use crate::services::{ServiceResult, WriterRef};

pub mod http;

pub fn writer_from_proto(proto_path: &PathBuf, addr: &str) -> ServiceResult<WriterRef> {
    todo!()
}
