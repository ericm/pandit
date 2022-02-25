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
//! Generated file from `examples/example1.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_27_1;

#[derive(PartialEq,Clone,Default)]
pub struct ExampleRequest {
    // message fields
    pub id: i32,
    pub user: ::std::string::String,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a ExampleRequest {
    fn default() -> &'a ExampleRequest {
        <ExampleRequest as ::protobuf::Message>::default_instance()
    }
}

impl ExampleRequest {
    pub fn new() -> ExampleRequest {
        ::std::default::Default::default()
    }

    // int32 id = 1;


    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn clear_id(&mut self) {
        self.id = 0;
    }

    // Param is passed by value, moved
    pub fn set_id(&mut self, v: i32) {
        self.id = v;
    }

    // string user = 2;


    pub fn get_user(&self) -> &str {
        &self.user
    }
    pub fn clear_user(&mut self) {
        self.user.clear();
    }

    // Param is passed by value, moved
    pub fn set_user(&mut self, v: ::std::string::String) {
        self.user = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_user(&mut self) -> &mut ::std::string::String {
        &mut self.user
    }

    // Take field
    pub fn take_user(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.user, ::std::string::String::new())
    }
}

impl ::protobuf::Message for ExampleRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.id = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.user)?;
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
        if self.id != 0 {
            my_size += ::protobuf::rt::value_size(1, self.id, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.user.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.user);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.id != 0 {
            os.write_int32(1, self.id)?;
        }
        if !self.user.is_empty() {
            os.write_string(2, &self.user)?;
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

    fn new() -> ExampleRequest {
        ExampleRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "id",
                |m: &ExampleRequest| { &m.id },
                |m: &mut ExampleRequest| { &mut m.id },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "user",
                |m: &ExampleRequest| { &m.user },
                |m: &mut ExampleRequest| { &mut m.user },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<ExampleRequest>(
                "ExampleRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static ExampleRequest {
        static instance: ::protobuf::rt::LazyV2<ExampleRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(ExampleRequest::new)
    }
}

impl ::protobuf::Clear for ExampleRequest {
    fn clear(&mut self) {
        self.id = 0;
        self.user.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ExampleRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ExampleRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ExampleResponse {
    // message fields
    pub id: i32,
    pub user: ::std::string::String,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a ExampleResponse {
    fn default() -> &'a ExampleResponse {
        <ExampleResponse as ::protobuf::Message>::default_instance()
    }
}

impl ExampleResponse {
    pub fn new() -> ExampleResponse {
        ::std::default::Default::default()
    }

    // int32 id = 1;


    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn clear_id(&mut self) {
        self.id = 0;
    }

    // Param is passed by value, moved
    pub fn set_id(&mut self, v: i32) {
        self.id = v;
    }

    // string user = 2;


    pub fn get_user(&self) -> &str {
        &self.user
    }
    pub fn clear_user(&mut self) {
        self.user.clear();
    }

    // Param is passed by value, moved
    pub fn set_user(&mut self, v: ::std::string::String) {
        self.user = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_user(&mut self) -> &mut ::std::string::String {
        &mut self.user
    }

    // Take field
    pub fn take_user(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.user, ::std::string::String::new())
    }
}

impl ::protobuf::Message for ExampleResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.id = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.user)?;
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
        if self.id != 0 {
            my_size += ::protobuf::rt::value_size(1, self.id, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.user.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.user);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.id != 0 {
            os.write_int32(1, self.id)?;
        }
        if !self.user.is_empty() {
            os.write_string(2, &self.user)?;
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

    fn new() -> ExampleResponse {
        ExampleResponse::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "id",
                |m: &ExampleResponse| { &m.id },
                |m: &mut ExampleResponse| { &mut m.id },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "user",
                |m: &ExampleResponse| { &m.user },
                |m: &mut ExampleResponse| { &mut m.user },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<ExampleResponse>(
                "ExampleResponse",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static ExampleResponse {
        static instance: ::protobuf::rt::LazyV2<ExampleResponse> = ::protobuf::rt::LazyV2::INIT;
        instance.get(ExampleResponse::new)
    }
}

impl ::protobuf::Clear for ExampleResponse {
    fn clear(&mut self) {
        self.id = 0;
        self.user.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ExampleResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ExampleResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x17examples/example1.proto\x12\thttp_demo\x1a\x0cpandit.proto\x1a\x11\
    format/http.proto\x1a\rhandler.proto\":\n\x0eExampleRequest\x12\x14\n\
    \x02id\x18\x01\x20\x01(\x05R\x02idB\x04\xa8\xb7\x18\x01\x12\x12\n\x04use\
    r\x18\x02\x20\x01(\tR\x04user\"?\n\x0fExampleResponse\x12\x0e\n\x02id\
    \x18\x01\x20\x01(\x05R\x02id\x12\x12\n\x04user\x18\x02\x20\x01(\tR\x04us\
    er:\x08\xf2\xb6\x18\x04.obj2\x9a\x01\n\x0eExampleService\x12b\n\nGetExam\
    ple\x12\x19.http_demo.ExampleRequest\x1a\x1a.http_demo.ExampleResponse\"\
    \x1d\x98\xb8\x18\0\xda\xb5\x18\x0c\x8a\xb5\x18\x08/example\x92\xb7\x18\
    \x05\x80\xa8\x1d\xb8\x17\x1a$\xe2\xb5\x18\x12\xb2\xb5\x18\n%hostname%\
    \xb8\xb5\x18\x01\xd2\xb5\x18\nmy_serviceb\x06proto3\
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