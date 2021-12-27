use crate::proto;
use protobuf::wire_format::WireType::WireTypeLengthDelimited;
use protobuf::{self};
use protobuf_parse;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::str;
use std::str::FromStr;
use std::{fmt::Display, path::PathBuf};

pub enum Protocols {
    HTTP,
}

pub mod http {
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
pub struct ServiceError {
    err: String,
}

impl ServiceError {
    fn new(err: String) -> Self {
        ServiceError { err }
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to create service object: {}", self.err)
    }
}

pub union MethodAPI {
    pub http: ManuallyDrop<http::API>,
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
    pub api: MethodAPI,
    pub input_message: protobuf::descriptor::MethodDescriptorProto,
    pub output_message: Message,
}

pub struct Service {
    pub name: String,
    pub methods: HashMap<String, MethodAPI>,
    pub messages: HashMap<String, Message>,
}

impl Service {
    pub fn from_file(path: &str) -> Result<Self, ServiceError> {
        let path_buf = &PathBuf::from(path);
        let include = PathBuf::from("./src/proto");
        let parsed =
            protobuf_parse::pure::parse_and_typecheck(&[include], &[path_buf.clone()]).unwrap();
        let filename = path_buf.file_name().unwrap();
        let file = parsed
            .file_descriptors
            .iter()
            .find(|&x| x.get_name() == filename)
            .unwrap();
        let service = file.service.first().unwrap();

        let mut output = Self::default();
        output.get_service_attrs_base(file)?;
        match Self::get_service_type(service).unwrap() {
            Protocols::HTTP => output.get_service_attrs_http(service)?,
        };

        Ok(output)
    }

    fn get_service_type(
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Option<Protocols> {
        if proto::http::exts::name
            .get(service.options.get_ref())
            .is_some()
        {
            Some(Protocols::HTTP)
        } else {
            None
        }
    }

    fn get_service_attrs_base(
        &mut self,
        file: &protobuf::descriptor::FileDescriptorProto,
    ) -> Result<(), ServiceError> {
        use proto::pandit::exts;
        self.messages = file
            .message_type
            .iter()
            .map(|message| {
                let mut config = Message::default();
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
                let mut config = MessageField::default();
                let name = field.get_name().to_string();
                config.proto = Box::new(field.clone());

                let opts = field.options.get_ref();
                config.absolute_path = exts::absolute_path.get(opts).unwrap();
                config.relative_path = exts::relative_path.get(opts).unwrap();

                (name, config)
            })
            .collect()
    }

    fn get_service_attrs_http(
        &mut self,
        service: &protobuf::descriptor::ServiceDescriptorProto,
    ) -> Result<(), ServiceError> {
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
