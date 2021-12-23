use protobuf;
use protobuf_parse;
use std::collections::HashSet;
use std::str::FromStr;
use std::{fmt::Display, path::PathBuf};

enum Protocols {
    HTTP,
}

impl FromStr for Protocols {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::HTTP),
            _ => Err(()),
        }
    }
}

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
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let path = PathBuf::from(path);
        let include = PathBuf::from("./src/config/proto");
        let parsed = match protobuf_parse::pure::parse_and_typecheck(&[include], &[path]) {
            Ok(p) => p,
            Err(err) => return Err(ConfigError::new(err.to_string())),
        };
        let file = parsed.file_descriptors.first().unwrap();
        let service = file.service.first().unwrap();
        let opts = service.options.as_ref().unwrap();
        for opt in opts.uninterpreted_option.iter() {
            println!("a{}", opt.name.first().unwrap().get_name_part());
        }
        Ok(Service::new())
    }
}

#[test]
fn test_service() {
    Service::from_file("./src/config/proto/example.proto").unwrap();
}
