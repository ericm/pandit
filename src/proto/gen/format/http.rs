// This file is generated by rust-protobuf 3.0.0-alpha.6. Do not edit
// .proto file is parsed by protoc --rust-out=...
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
#![allow(unused_results)]
#![allow(unused_mut)]

//! Generated file from `http.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_3_0_0_ALPHA_6;

#[derive(PartialEq,Clone,Default,Debug)]
// @@protoc_insertion_point(message:pandit.format.HTTP)
pub struct HTTP {
    // message fields
    // @@protoc_insertion_point(field:pandit.format.HTTP.content_type)
    pub content_type: ::std::string::String,
    // message oneof groups
    pub pattern: ::std::option::Option<http::Pattern>,
    // special fields
    // @@protoc_insertion_point(special_field:pandit.format.HTTP.unknown_fields)
    pub unknown_fields: ::protobuf::UnknownFields,
    // @@protoc_insertion_point(special_field:pandit.format.HTTP.cached_size)
    pub cached_size: ::protobuf::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a HTTP {
    fn default() -> &'a HTTP {
        <HTTP as ::protobuf::Message>::default_instance()
    }
}

impl HTTP {
    pub fn new() -> HTTP {
        ::std::default::Default::default()
    }

    // string get = 50001;

    pub fn get_get(&self) -> &str {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::get(ref v)) => v,
            _ => "",
        }
    }

    pub fn clear_get(&mut self) {
        self.pattern = ::std::option::Option::None;
    }

    pub fn has_get(&self) -> bool {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::get(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_get(&mut self, v: ::std::string::String) {
        self.pattern = ::std::option::Option::Some(http::Pattern::get(v))
    }

    // Mutable pointer to the field.
    pub fn mut_get(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(http::Pattern::get(_)) = self.pattern {
        } else {
            self.pattern = ::std::option::Option::Some(http::Pattern::get(::std::string::String::new()));
        }
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::get(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_get(&mut self) -> ::std::string::String {
        if self.has_get() {
            match self.pattern.take() {
                ::std::option::Option::Some(http::Pattern::get(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    // string put = 50002;

    pub fn get_put(&self) -> &str {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::put(ref v)) => v,
            _ => "",
        }
    }

    pub fn clear_put(&mut self) {
        self.pattern = ::std::option::Option::None;
    }

    pub fn has_put(&self) -> bool {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::put(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_put(&mut self, v: ::std::string::String) {
        self.pattern = ::std::option::Option::Some(http::Pattern::put(v))
    }

    // Mutable pointer to the field.
    pub fn mut_put(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(http::Pattern::put(_)) = self.pattern {
        } else {
            self.pattern = ::std::option::Option::Some(http::Pattern::put(::std::string::String::new()));
        }
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::put(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_put(&mut self) -> ::std::string::String {
        if self.has_put() {
            match self.pattern.take() {
                ::std::option::Option::Some(http::Pattern::put(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    // string post = 50003;

    pub fn get_post(&self) -> &str {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::post(ref v)) => v,
            _ => "",
        }
    }

    pub fn clear_post(&mut self) {
        self.pattern = ::std::option::Option::None;
    }

    pub fn has_post(&self) -> bool {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::post(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_post(&mut self, v: ::std::string::String) {
        self.pattern = ::std::option::Option::Some(http::Pattern::post(v))
    }

    // Mutable pointer to the field.
    pub fn mut_post(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(http::Pattern::post(_)) = self.pattern {
        } else {
            self.pattern = ::std::option::Option::Some(http::Pattern::post(::std::string::String::new()));
        }
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::post(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_post(&mut self) -> ::std::string::String {
        if self.has_post() {
            match self.pattern.take() {
                ::std::option::Option::Some(http::Pattern::post(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    // string delete = 50004;

    pub fn get_delete(&self) -> &str {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::delete(ref v)) => v,
            _ => "",
        }
    }

    pub fn clear_delete(&mut self) {
        self.pattern = ::std::option::Option::None;
    }

    pub fn has_delete(&self) -> bool {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::delete(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_delete(&mut self, v: ::std::string::String) {
        self.pattern = ::std::option::Option::Some(http::Pattern::delete(v))
    }

    // Mutable pointer to the field.
    pub fn mut_delete(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(http::Pattern::delete(_)) = self.pattern {
        } else {
            self.pattern = ::std::option::Option::Some(http::Pattern::delete(::std::string::String::new()));
        }
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::delete(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_delete(&mut self) -> ::std::string::String {
        if self.has_delete() {
            match self.pattern.take() {
                ::std::option::Option::Some(http::Pattern::delete(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    // string patch = 50005;

    pub fn get_patch(&self) -> &str {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::patch(ref v)) => v,
            _ => "",
        }
    }

    pub fn clear_patch(&mut self) {
        self.pattern = ::std::option::Option::None;
    }

    pub fn has_patch(&self) -> bool {
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::patch(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_patch(&mut self, v: ::std::string::String) {
        self.pattern = ::std::option::Option::Some(http::Pattern::patch(v))
    }

    // Mutable pointer to the field.
    pub fn mut_patch(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(http::Pattern::patch(_)) = self.pattern {
        } else {
            self.pattern = ::std::option::Option::Some(http::Pattern::patch(::std::string::String::new()));
        }
        match self.pattern {
            ::std::option::Option::Some(http::Pattern::patch(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_patch(&mut self) -> ::std::string::String {
        if self.has_patch() {
            match self.pattern.take() {
                ::std::option::Option::Some(http::Pattern::patch(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(6);
        fields.push(::protobuf::reflect::rt::v2::make_simpler_field_accessor::<_, _>(
            "content_type",
            |m: &HTTP| { &m.content_type },
            |m: &mut HTTP| { &mut m.content_type },
        ));
        fields.push(::protobuf::reflect::rt::v2::make_oneof_deref_has_get_set_simpler_accessor::<_, _>(
            "get",
            HTTP::has_get,
            HTTP::get_get,
            HTTP::set_get,
        ));
        fields.push(::protobuf::reflect::rt::v2::make_oneof_deref_has_get_set_simpler_accessor::<_, _>(
            "put",
            HTTP::has_put,
            HTTP::get_put,
            HTTP::set_put,
        ));
        fields.push(::protobuf::reflect::rt::v2::make_oneof_deref_has_get_set_simpler_accessor::<_, _>(
            "post",
            HTTP::has_post,
            HTTP::get_post,
            HTTP::set_post,
        ));
        fields.push(::protobuf::reflect::rt::v2::make_oneof_deref_has_get_set_simpler_accessor::<_, _>(
            "delete",
            HTTP::has_delete,
            HTTP::get_delete,
            HTTP::set_delete,
        ));
        fields.push(::protobuf::reflect::rt::v2::make_oneof_deref_has_get_set_simpler_accessor::<_, _>(
            "patch",
            HTTP::has_patch,
            HTTP::get_patch,
            HTTP::set_patch,
        ));
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<HTTP>(
            "HTTP",
            0,
            fields,
        )
    }
}

impl ::protobuf::Message for HTTP {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                50000 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.content_type = is.read_string()?;
                },
                50001 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.pattern = ::std::option::Option::Some(http::Pattern::get(is.read_string()?));
                },
                50002 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.pattern = ::std::option::Option::Some(http::Pattern::put(is.read_string()?));
                },
                50003 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.pattern = ::std::option::Option::Some(http::Pattern::post(is.read_string()?));
                },
                50004 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.pattern = ::std::option::Option::Some(http::Pattern::delete(is.read_string()?));
                },
                50005 => {
                    if wire_type != ::protobuf::rt::WireType::LengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    self.pattern = ::std::option::Option::Some(http::Pattern::patch(is.read_string()?));
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
        if !self.content_type.is_empty() {
            my_size += ::protobuf::rt::string_size(50000, &self.content_type);
        }
        if let ::std::option::Option::Some(ref v) = self.pattern {
            match v {
                &http::Pattern::get(ref v) => {
                    my_size += ::protobuf::rt::string_size(50001, &v);
                },
                &http::Pattern::put(ref v) => {
                    my_size += ::protobuf::rt::string_size(50002, &v);
                },
                &http::Pattern::post(ref v) => {
                    my_size += ::protobuf::rt::string_size(50003, &v);
                },
                &http::Pattern::delete(ref v) => {
                    my_size += ::protobuf::rt::string_size(50004, &v);
                },
                &http::Pattern::patch(ref v) => {
                    my_size += ::protobuf::rt::string_size(50005, &v);
                },
            };
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if !self.content_type.is_empty() {
            os.write_string(50000, &self.content_type)?;
        }
        if let ::std::option::Option::Some(ref v) = self.pattern {
            match v {
                &http::Pattern::get(ref v) => {
                    os.write_string(50001, v)?;
                },
                &http::Pattern::put(ref v) => {
                    os.write_string(50002, v)?;
                },
                &http::Pattern::post(ref v) => {
                    os.write_string(50003, v)?;
                },
                &http::Pattern::delete(ref v) => {
                    os.write_string(50004, v)?;
                },
                &http::Pattern::patch(ref v) => {
                    os.write_string(50005, v)?;
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

    fn new() -> HTTP {
        HTTP::new()
    }

    fn descriptor_static() -> ::protobuf::reflect::MessageDescriptor {
        ::protobuf::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 0)
    }

    fn default_instance() -> &'static HTTP {
        static instance: HTTP = HTTP {
            content_type: ::std::string::String::new(),
            pattern: ::std::option::Option::None,
            unknown_fields: ::protobuf::UnknownFields::new(),
            cached_size: ::protobuf::rt::CachedSize::new(),
        };
        &instance
    }
}

impl ::protobuf::Clear for HTTP {
    fn clear(&mut self) {
        self.content_type.clear();
        self.pattern = ::std::option::Option::None;
        self.pattern = ::std::option::Option::None;
        self.pattern = ::std::option::Option::None;
        self.pattern = ::std::option::Option::None;
        self.pattern = ::std::option::Option::None;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Display for HTTP {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HTTP {
    type RuntimeType = ::protobuf::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

/// Nested message and enums of message `HTTP`
pub mod http {

    #[derive(Clone,PartialEq,Debug)]
    #[non_exhaustive]
    // @@protoc_insertion_point(oneof:pandit.format.HTTP.pattern)
    pub enum Pattern {
        // @@protoc_insertion_point(oneof_field:pandit.format.HTTP.get)
        get(::std::string::String),
        // @@protoc_insertion_point(oneof_field:pandit.format.HTTP.put)
        put(::std::string::String),
        // @@protoc_insertion_point(oneof_field:pandit.format.HTTP.post)
        post(::std::string::String),
        // @@protoc_insertion_point(oneof_field:pandit.format.HTTP.delete)
        delete(::std::string::String),
        // @@protoc_insertion_point(oneof_field:pandit.format.HTTP.patch)
        patch(::std::string::String),
    }

    impl ::protobuf::Oneof for Pattern {
    }
}

/// Extension fields
pub mod exts {

    pub const http: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::MethodOptions, ::protobuf::reflect::types::ProtobufTypeMessage<super::HTTP>> = ::protobuf::ext::ExtFieldOptional { field_number: 50011, phantom: ::std::marker::PhantomData };
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\nhttp.proto\x12\rpandit.format\x1a\x20google/protobuf/descriptor.prot\
    o\"\xb0\x01\n\x04HTTP\x12#\n\x0ccontent_type\x18\xd0\x86\x03\x20\x01(\tR\
    \x0bcontentType\x12\x14\n\x03get\x18\xd1\x86\x03\x20\x01(\tH\0R\x03get\
    \x12\x14\n\x03put\x18\xd2\x86\x03\x20\x01(\tH\0R\x03put\x12\x16\n\x04pos\
    t\x18\xd3\x86\x03\x20\x01(\tH\0R\x04post\x12\x1a\n\x06delete\x18\xd4\x86\
    \x03\x20\x01(\tH\0R\x06delete\x12\x18\n\x05patch\x18\xd5\x86\x03\x20\x01\
    (\tH\0R\x05patchB\t\n\x07pattern:I\n\x04http\x18\xdb\x86\x03\x20\x01(\
    \x0b2\x13.pandit.format.HTTP\x12\x1e.google.protobuf.MethodOptionsR\x04h\
    ttpJ\xa5\x06\n\x06\x12\x04\0\0\x1a;\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\t\
    \n\x02\x03\0\x12\x03\x01\0*\n\x08\n\x01\x02\x12\x03\x03\0\x16\n\n\n\x02\
    \x04\0\x12\x04\x05\0\x18\x01\n\n\n\x03\x04\0\x01\x12\x03\x05\x08\x0c\n\
    \x0b\n\x04\x04\0\x02\0\x12\x03\x06\x02\x1e\n\x0c\n\x05\x04\0\x02\0\x05\
    \x12\x03\x06\x02\x08\n\x0c\n\x05\x04\0\x02\0\x01\x12\x03\x06\t\x15\n\x0c\
    \n\x05\x04\0\x02\0\x03\x12\x03\x06\x18\x1d\n\x0c\n\x04\x04\0\x08\0\x12\
    \x04\x07\x02\x17\x03\n\x0c\n\x05\x04\0\x08\0\x01\x12\x03\x07\x08\x0f\n[\
    \n\x04\x04\0\x02\x01\x12\x03\n\x04\x17\x1aN\x20Maps\x20to\x20HTTP\x20GET\
    .\x20Used\x20for\x20listing\x20and\x20getting\x20information\x20about\n\
    \x20resources.\n\n\x0c\n\x05\x04\0\x02\x01\x05\x12\x03\n\x04\n\n\x0c\n\
    \x05\x04\0\x02\x01\x01\x12\x03\n\x0b\x0e\n\x0c\n\x05\x04\0\x02\x01\x03\
    \x12\x03\n\x11\x16\n?\n\x04\x04\0\x02\x02\x12\x03\r\x04\x17\x1a2\x20Maps\
    \x20to\x20HTTP\x20PUT.\x20Used\x20for\x20replacing\x20a\x20resource.\n\n\
    \x0c\n\x05\x04\0\x02\x02\x05\x12\x03\r\x04\n\n\x0c\n\x05\x04\0\x02\x02\
    \x01\x12\x03\r\x0b\x0e\n\x0c\n\x05\x04\0\x02\x02\x03\x12\x03\r\x11\x16\n\
    W\n\x04\x04\0\x02\x03\x12\x03\x10\x04\x18\x1aJ\x20Maps\x20to\x20HTTP\x20\
    POST.\x20Used\x20for\x20creating\x20a\x20resource\x20or\x20performing\
    \x20an\x20action.\n\n\x0c\n\x05\x04\0\x02\x03\x05\x12\x03\x10\x04\n\n\
    \x0c\n\x05\x04\0\x02\x03\x01\x12\x03\x10\x0b\x0f\n\x0c\n\x05\x04\0\x02\
    \x03\x03\x12\x03\x10\x12\x17\nA\n\x04\x04\0\x02\x04\x12\x03\x13\x04\x1a\
    \x1a4\x20Maps\x20to\x20HTTP\x20DELETE.\x20Used\x20for\x20deleting\x20a\
    \x20resource.\n\n\x0c\n\x05\x04\0\x02\x04\x05\x12\x03\x13\x04\n\n\x0c\n\
    \x05\x04\0\x02\x04\x01\x12\x03\x13\x0b\x11\n\x0c\n\x05\x04\0\x02\x04\x03\
    \x12\x03\x13\x14\x19\n@\n\x04\x04\0\x02\x05\x12\x03\x16\x04\x19\x1a3\x20\
    Maps\x20to\x20HTTP\x20PATCH.\x20Used\x20for\x20updating\x20a\x20resource\
    .\n\n\x0c\n\x05\x04\0\x02\x05\x05\x12\x03\x16\x04\n\n\x0c\n\x05\x04\0\
    \x02\x05\x01\x12\x03\x16\x0b\x10\n\x0c\n\x05\x04\0\x02\x05\x03\x12\x03\
    \x16\x13\x18\n\x08\n\x01\x07\x12\x03\x1a\0;\n\t\n\x02\x07\0\x12\x03\x1a'\
    9\n\n\n\x03\x07\0\x02\x12\x03\x1a\x07$\n\n\n\x03\x07\0\x06\x12\x03\x1a'+\
    \n\n\n\x03\x07\0\x01\x12\x03\x1a,0\n\n\n\x03\x07\0\x03\x12\x03\x1a38b\
    \x06proto3\
";

/// `FileDescriptorProto` object which was a source for this generated file
pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;
    file_descriptor_proto_lazy.get(|| {
        ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
    })
}

/// `FileDescriptor` object which allows dynamic access to files
pub fn file_descriptor() -> ::protobuf::reflect::FileDescriptor {
    static file_descriptor_lazy: ::protobuf::rt::LazyV2<::protobuf::reflect::GeneratedFileDescriptor> = ::protobuf::rt::LazyV2::INIT;
    let file_descriptor = file_descriptor_lazy.get(|| {
        let mut deps = ::std::vec::Vec::new();
        deps.push(::protobuf::descriptor::file_descriptor());
        let mut messages = ::std::vec::Vec::new();
        messages.push(HTTP::generated_message_descriptor_data());
        let mut enums = ::std::vec::Vec::new();
        ::protobuf::reflect::GeneratedFileDescriptor::new_generated(
            file_descriptor_proto(),
            deps,
            messages,
            enums,
        )
    });
    ::protobuf::reflect::FileDescriptor::new_generated_2(file_descriptor)
}
