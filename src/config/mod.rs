use crate::proto;
use protobuf::wire_format::WireType::WireTypeLengthDelimited;
use protobuf::{self, Message};
use protobuf_parse;
use std::collections::HashSet;
use std::str;
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
        let opts = service.options.get_ref();
        let opts = Service::from_service_options(opts);
        println!("{:?}", opts);

        let methods = &service.method;
        for method in methods {
            let opt = method.options.0.as_ref().unwrap();
            println!("{:?}", opt);
            println!("{:?}", opt.unknown_fields.get(50011).unwrap());
        }
        Ok(Service::new())
    }

    fn from_service_options(opts: &protobuf::descriptor::ServiceOptions) -> Vec<String> {
        let p = proto::http::exts::name.get(opts).unwrap();
        opts.get_unknown_fields()
            .iter()
            .map(|(_, val)| Service::from_unknown_value_ref(val).unwrap())
            .collect()
    }

    fn from_unknown_value_ref(
        val: &protobuf::UnknownValues,
    ) -> Result<String, std::string::FromUtf8Error> {
        let del_val = val.length_delimited.first().unwrap();
        String::from_utf8(del_val.to_vec())
    }
}

#[test]
fn test_service() {
    Service::from_file("./src/proto/example.proto").unwrap();
}
