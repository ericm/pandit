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
//! Generated file from `examples/factorial.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_27_1;

#[derive(PartialEq,Clone,Default)]
pub struct FactorialRequest {
    // message fields
    pub number: i32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a FactorialRequest {
    fn default() -> &'a FactorialRequest {
        <FactorialRequest as ::protobuf::Message>::default_instance()
    }
}

impl FactorialRequest {
    pub fn new() -> FactorialRequest {
        ::std::default::Default::default()
    }

    // int32 number = 1;


    pub fn get_number(&self) -> i32 {
        self.number
    }
    pub fn clear_number(&mut self) {
        self.number = 0;
    }

    // Param is passed by value, moved
    pub fn set_number(&mut self, v: i32) {
        self.number = v;
    }
}

impl ::protobuf::Message for FactorialRequest {
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
                    self.number = tmp;
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
        if self.number != 0 {
            my_size += ::protobuf::rt::value_size(1, self.number, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.number != 0 {
            os.write_int32(1, self.number)?;
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

    fn new() -> FactorialRequest {
        FactorialRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "number",
                |m: &FactorialRequest| { &m.number },
                |m: &mut FactorialRequest| { &mut m.number },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<FactorialRequest>(
                "FactorialRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static FactorialRequest {
        static instance: ::protobuf::rt::LazyV2<FactorialRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(FactorialRequest::new)
    }
}

impl ::protobuf::Clear for FactorialRequest {
    fn clear(&mut self) {
        self.number = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FactorialRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FactorialRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct FactorialResponse {
    // message fields
    pub response: i32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a FactorialResponse {
    fn default() -> &'a FactorialResponse {
        <FactorialResponse as ::protobuf::Message>::default_instance()
    }
}

impl FactorialResponse {
    pub fn new() -> FactorialResponse {
        ::std::default::Default::default()
    }

    // int32 response = 1;


    pub fn get_response(&self) -> i32 {
        self.response
    }
    pub fn clear_response(&mut self) {
        self.response = 0;
    }

    // Param is passed by value, moved
    pub fn set_response(&mut self, v: i32) {
        self.response = v;
    }
}

impl ::protobuf::Message for FactorialResponse {
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
                    self.response = tmp;
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
        if self.response != 0 {
            my_size += ::protobuf::rt::value_size(1, self.response, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.response != 0 {
            os.write_int32(1, self.response)?;
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

    fn new() -> FactorialResponse {
        FactorialResponse::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                "response",
                |m: &FactorialResponse| { &m.response },
                |m: &mut FactorialResponse| { &mut m.response },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<FactorialResponse>(
                "FactorialResponse",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static FactorialResponse {
        static instance: ::protobuf::rt::LazyV2<FactorialResponse> = ::protobuf::rt::LazyV2::INIT;
        instance.get(FactorialResponse::new)
    }
}

impl ::protobuf::Clear for FactorialResponse {
    fn clear(&mut self) {
        self.response = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for FactorialResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for FactorialResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x18examples/factorial.proto\x12\tfactorial\x1a\x0cpandit.proto\x1a\
    \x11format/http.proto\x1a\rhandler.proto\"0\n\x10FactorialRequest\x12\
    \x1c\n\x06number\x18\x01\x20\x01(\x05R\x06numberB\x04\xa8\xb7\x18\x01\"/\
    \n\x11FactorialResponse\x12\x1a\n\x08response\x18\x01\x20\x01(\x05R\x08r\
    esponse2\xa2\x01\n\x10FactorialService\x12j\n\x0cGetFactorial\x12\x1b.fa\
    ctorial.FactorialRequest\x1a\x1c.factorial.FactorialResponse\"\x1f\x98\
    \xb8\x18\0\xda\xb5\x18\x0e\x9a\xb5\x18\n/factorial\x92\xb7\x18\x05\x80\
    \xa8\x1d\xb8\x17\x1a\"\xe2\xb5\x18\x11\xb2\xb5\x18\tlocalhost\xb8\xb5\
    \x18\x01\xd2\xb5\x18\tfactorialb\x06proto3\
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
