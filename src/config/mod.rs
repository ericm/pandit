use crate::proto;
use protobuf::wire_format::WireType::WireTypeLengthDelimited;
use protobuf::{self, Message};
use protobuf_parse;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::str;
use std::str::FromStr;
use std::{fmt::Display, path::PathBuf};

enum Protocols {
    HTTP,
}

mod http {
    pub use crate::proto::http::api;
    pub use crate::proto::http::API;
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

union MethodAPI {
    http: ManuallyDrop<http::API>,
}

pub struct Service {
    pub name: String,
    pub methods: HashMap<String, MethodAPI>,
}

impl Service {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let path_buf = &PathBuf::from(path);
        let include = PathBuf::from("./src/proto");
        let parsed =
            match protobuf_parse::pure::parse_and_typecheck(&[include], &[path_buf.clone()]) {
                Ok(p) => p,
                Err(err) => return Err(ConfigError::new(err.to_string())),
            };
        let filename = path_buf.file_name().unwrap();
        let file = parsed
            .file_descriptors
            .iter()
            .find(|&x| x.get_name() == filename)
            .unwrap();
        let service = file.service.first().unwrap();
        println!("{}", p);
        Ok(Service::new())
    }
    fn get_service_attrs_http(
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<Self, ConfigError> {
        use proto::http::exts;
        let config = Self::default();

        let opts = service.options.get_ref();
        config.name = exts::name.get(opts).unwrap();

        config.methods = service
            .method
            .iter()
            .map(|method| {
                (
                    method.get_name().to_string(),
                    MethodAPI {
                        http: ManuallyDrop::new(exts::api.get(method.options.get_ref()).unwrap()),
                    },
                )
            })
            .collect();

        Ok(config)
    }
}

impl Default for Service {
    fn default() -> Self {
        Self {
            name: String::default(),
            methods: HashMap::new(),
        }
    }
}

#[test]
fn test_service() {
    Service::from_file("./src/proto/example.proto").unwrap();
}
