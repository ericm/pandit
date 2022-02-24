// This file is generated by rust-protobuf 2.25.2. Do not edit
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
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

#[derive(PartialEq,Clone,Default)]
pub struct StartServiceRequest {
    // message fields
    pub name: ::std::string::String,
    pub proto: ::std::vec::Vec<u8>,
    pub addr: ::std::string::String,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a StartServiceRequest {
    fn default() -> &'a StartServiceRequest {
        <StartServiceRequest as ::protobuf::Message>::default_instance()
    }
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

    // string addr = 3;


    pub fn get_addr(&self) -> &str {
        &self.addr
    }
    pub fn clear_addr(&mut self) {
        self.addr.clear();
    }

    // Param is passed by value, moved
    pub fn set_addr(&mut self, v: ::std::string::String) {
        self.addr = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_addr(&mut self) -> &mut ::std::string::String {
        &mut self.addr
    }

    // Take field
    pub fn take_addr(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.addr, ::std::string::String::new())
    }
}

impl ::protobuf::Message for StartServiceRequest {
    fn is_initialized(&self) -> bool {
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
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.addr)?;
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
        if !self.addr.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.addr);
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
        if !self.addr.is_empty() {
            os.write_string(3, &self.addr)?;
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
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "addr",
                |m: &StartServiceRequest| { &m.addr },
                |m: &mut StartServiceRequest| { &mut m.addr },
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
        self.addr.clear();
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
    \n\tapi.proto\x12\x03api\"S\n\x13StartServiceRequest\x12\x12\n\x04name\
    \x18\x01\x20\x01(\tR\x04name\x12\x14\n\x05proto\x18\x02\x20\x01(\x0cR\
    \x05proto\x12\x12\n\x04addr\x18\x03\x20\x01(\tR\x04addr\"\x13\n\x11Start\
    ServiceReply2I\n\x03API\x12B\n\x0cStartService\x12\x18.api.StartServiceR\
    equest\x1a\x16.api.StartServiceReply\"\0b\x06proto3\
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
