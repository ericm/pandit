use protobuf;
use protobuf_parse;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub struct ConfigError {
    err: String,
}

impl ConfigError {
    fn new(err: String) -> Self {
        ConfigError { err }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to create config object: {}", self.err)
    }
}

pub struct Service {}

impl Service {
    pub fn new() -> Self {
        Service {}
    }
    pub fn from_file(path: String) -> Result<Self, ConfigError> {
        let path = PathBuf::from(path);
        let parsed = match protobuf_parse::pure::parse_and_typecheck(&[], &[path]) {
            Ok(p) => p,
            Err(err) => return Err(ConfigError::new(err.to_string())),
        };
        for file in parsed.file_descriptors {
            for message in file.message_type {}
        }
        Ok(Service::new())
    }
}
