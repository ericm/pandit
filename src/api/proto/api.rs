// This file is generated by rust-protobuf 2.27.1. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `api.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_27_1;

#[derive(PartialEq,Clone,Default)]
pub struct DockerNetwork {
    // message fields
    pub container_id: ::std::string::String,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a DockerNetwork {
    fn default() -> &'a DockerNetwork {
        <DockerNetwork as ::protobuf::Message>::default_instance()
    }
}

impl DockerNetwork {
    pub fn new() -> DockerNetwork {
        ::std::default::Default::default()
    }

    // string container_id = 1;


    pub fn get_container_id(&self) -> &str {
        &self.container_id
    }
    pub fn clear_container_id(&mut self) {
        self.container_id.clear();
    }

    // Param is passed by value, moved
    pub fn set_container_id(&mut self, v: ::std::string::String) {
        self.container_id = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_container_id(&mut self) -> &mut ::std::string::String {
        &mut self.container_id
    }

    // Take field
    pub fn take_container_id(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.container_id, ::std::string::String::new())
    }
}

impl ::protobuf::Message for DockerNetwork {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.container_id)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.container_id.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.container_id);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.container_id.is_empty() {
            os.write_string(1, &self.container_id)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> DockerNetwork {
        DockerNetwork::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "container_id",
                |m: &DockerNetwork| { &m.container_id },
                |m: &mut DockerNetwork| { &mut m.container_id },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<DockerNetwork>(
                "DockerNetwork",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static DockerNetwork {
        static instance: ::protobuf::rt::LazyV2<DockerNetwork> = ::protobuf::rt::LazyV2::INIT;
        instance.get(DockerNetwork::new)
    }
}

impl ::protobuf::Clear for DockerNetwork {
    fn clear(&mut self) {
        self.container_id.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DockerNetwork {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DockerNetwork {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct StartServiceRequest {
    // message fields
    pub name: ::std::string::String,
    pub proto: ::std::vec::Vec<u8>,
    pub port: i32,
    // message oneof groups
    pub network: ::std::option::Option<StartServiceRequest_oneof_network>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a StartServiceRequest {
    fn default() -> &'a StartServiceRequest {
        <StartServiceRequest as ::protobuf::Message>::default_instance()
    }
}

#[derive(Clone,PartialEq,Debug)]
pub enum StartServiceRequest_oneof_network {
    docker_network(DockerNetwork),
}

impl StartServiceRequest {
    pub fn new() -> StartServiceRequest {
        ::std::default::Default::default()
    }

    // string name = 1;


    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    // bytes proto = 2;


    pub fn get_proto(&self) -> &[u8] {
        &self.proto
    }
    pub fn clear_proto(&mut self) {
        self.proto.clear();
    }

    // Param is passed by value, moved
    pub fn set_proto(&mut self, v: ::std::vec::Vec<u8>) {
        self.proto = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_proto(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.proto
    }

    // Take field
    pub fn take_proto(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.proto, ::std::vec::Vec::new())
    }

    // int32 port = 3;


    pub fn get_port(&self) -> i32 {
        self.port
    }
    pub fn clear_port(&mut self) {
        self.port = 0;
    }

    // Param is passed by value, moved
    pub fn set_port(&mut self, v: i32) {
        self.port = v;
    }

    // .api.DockerNetwork docker_network = 4;


    pub fn get_docker_network(&self) -> &DockerNetwork {
        match self.network {
            ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(ref v)) => v,
            _ => <DockerNetwork as ::protobuf::Message>::default_instance(),
        }
    }
    pub fn clear_docker_network(&mut self) {
        self.network = ::std::option::Option::None;
    }

    pub fn has_docker_network(&self) -> bool {
        match self.network {
            ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_docker_network(&mut self, v: DockerNetwork) {
        self.network = ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(v))
    }

    // Mutable pointer to the field.
    pub fn mut_docker_network(&mut self) -> &mut DockerNetwork {
        if let ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(_)) = self.network {
        } else {
            self.network = ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(DockerNetwork::new()));
        }
        match self.network {
            ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_docker_network(&mut self) -> DockerNetwork {
        if self.has_docker_network() {
            match self.network.take() {
                ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(v)) => v,
                _ => panic!(),
            }
        } else {
            DockerNetwork::new()
        }
    }
}

impl ::protobuf::Message for StartServiceRequest {
    fn is_initialized(&self) -> bool {
        if let Some(StartServiceRequest_oneof_network::docker_network(ref v)) = self.network {
            if !v.is_initialized() {
                return false;
            }
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.proto)?;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.port = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.network = ::std::option::Option::Some(StartServiceRequest_oneof_network::docker_network(is.read_message()?));
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        if !self.proto.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.proto);
        }
        if self.port != 0 {
            my_size += ::protobuf::rt::value_size(3, self.port, ::protobuf::wire_format::WireTypeVarint);
        }
        if let ::std::option::Option::Some(ref v) = self.network {
            match v {
                &StartServiceRequest_oneof_network::docker_network(ref v) => {
                    let len = v.compute_size();
                    my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
                },
            };
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        if !self.proto.is_empty() {
            os.write_bytes(2, &self.proto)?;
        }
        if self.port != 0 {
            os.write_int32(3, self.port)?;
        }
        if let ::std::option::Option::Some(ref v) = self.network {
            match v {
                &StartServiceRequest_oneof_network::docker_network(ref v) => {
                    os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
                    os.write_raw_varint32(v.get_cached_size())?;
                    v.write_to_with_cached_sizes(os)?;
                },
            };
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> StartServiceRequest {
        StartServiceRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "name",
                |m: &StartServiceRequest| { &m.name },
                |m: &mut StartServiceRequest| { &mut m.name },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "proto",
                |m: &StartServiceRequest| { &m.proto },
                |m: &mut StartServiceRequest| { &mut m.proto },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "port",
                |m: &StartServiceRequest| { &m.port },
                |m: &mut StartServiceRequest| { &mut m.port },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_message_accessor::<_, DockerNetwork>(
                "docker_network",
                StartServiceRequest::has_docker_network,
                StartServiceRequest::get_docker_network,
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<StartServiceRequest>(
                "StartServiceRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static StartServiceRequest {
        static instance: ::protobuf::rt::LazyV2<StartServiceRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(StartServiceRequest::new)
    }
}

impl ::protobuf::Clear for StartServiceRequest {
    fn clear(&mut self) {
        self.name.clear();
        self.proto.clear();
        self.port = 0;
        self.network = ::std::option::Option::None;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for StartServiceRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for StartServiceRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct StartServiceReply {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a StartServiceReply {
    fn default() -> &'a StartServiceReply {
        <StartServiceReply as ::protobuf::Message>::default_instance()
    }
}

impl StartServiceReply {
    pub fn new() -> StartServiceReply {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for StartServiceReply {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> StartServiceReply {
        StartServiceReply::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let fields = ::std::vec::Vec::new();
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<StartServiceReply>(
                "StartServiceReply",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static StartServiceReply {
        static instance: ::protobuf::rt::LazyV2<StartServiceReply> = ::protobuf::rt::LazyV2::INIT;
        instance.get(StartServiceReply::new)
    }
}

impl ::protobuf::Clear for StartServiceReply {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for StartServiceReply {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for StartServiceReply {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\tapi.proto\x12\x03api\"2\n\rDockerNetwork\x12!\n\x0ccontainer_id\x18\
    \x01\x20\x01(\tR\x0bcontainerId\"\x9b\x01\n\x13StartServiceRequest\x12\
    \x12\n\x04name\x18\x01\x20\x01(\tR\x04name\x12\x14\n\x05proto\x18\x02\
    \x20\x01(\x0cR\x05proto\x12\x12\n\x04port\x18\x03\x20\x01(\x05R\x04port\
    \x12;\n\x0edocker_network\x18\x04\x20\x01(\x0b2\x12.api.DockerNetworkH\0\
    R\rdockerNetworkB\t\n\x07network\"\x13\n\x11StartServiceReply2I\n\x03API\
    \x12B\n\x0cStartService\x12\x18.api.StartServiceRequest\x1a\x16.api.Star\
    tServiceReply\"\0b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
