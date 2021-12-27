use crate::proto;
use protobuf::wire_format::WireType::WireTypeLengthDelimited;
use protobuf::{self};
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

pub union MethodAPI {
    http: ManuallyDrop<http::API>,
}

pub struct MessageField {
    proto: Box<protobuf::descriptor::FieldDescriptorProto>,
    absolute_path: String,
    relative_path: String,
}

impl Default for MessageField {
    fn default() -> Self {
        Self {
            proto: Default::default(),
            absolute_path: Default::default(),
            relative_path: Default::default(),
        }
    }
}
pub struct Message {
    path: String,
    fields: HashMap<String, MessageField>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            path: Default::default(),
            fields: Default::default(),
        }
    }
}

pub struct Method {
    api: MethodAPI,
    input_message: protobuf::descriptor::MethodDescriptorProto,
    output_message: Message,
}

pub struct Service {
    pub name: String,
    pub methods: HashMap<String, MethodAPI>,
    pub messages: HashMap<String, Message>,
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

    fn get_service_attrs_base(
        &self,
        file: &protobuf::descriptor::FileDescriptorProto,
    ) -> Result<(), ConfigError> {
        use proto::pandit::exts;
        let messages: HashMap<_, _> = file
            .message_type
            .iter()
            .map(|message| {
                let config = Message::default();
                let name = message.get_name().to_string();
                let opts = message.options.get_ref();
                config.path = exts::path.get(opts).unwrap();
                config.fields = Self::get_message_field_attrs(&message);
                (name, config)
            })
            .collect();
        Ok(())
    }

    fn get_message_field_attrs(
        message: &protobuf::descriptor::DescriptorProto,
    ) -> HashMap<String, MessageField> {
        use proto::pandit::exts;
        message
            .field
            .iter()
            .map(|field| {
                let config = MessageField::default();
                let name = field.get_name().to_string();
                config.proto = Box::new(*field);

                let opts = field.options.get_ref();
                config.absolute_path = exts::absolute_path.get(opts).unwrap();
                config.relative_path = exts::relative_path.get(opts).unwrap();

                (name, config)
            })
            .collect()
    }

    fn get_service_attrs_http(
        &self,
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<(), ConfigError> {
        use proto::http::exts;

        let opts = service.options.get_ref();
        self.name = exts::name.get(opts).unwrap();

        self.methods = service
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

        Ok(())
    }
}

impl Default for Service {
    fn default() -> Self {
        Self {
            name: Default::default(),
            methods: Default::default(),
            messages: Default::default(),
        }
    }
}

#[test]
fn test_service() {
    Service::from_file("./src/proto/example.proto").unwrap();
}
