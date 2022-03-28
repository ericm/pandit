// This file is generated by rust-protobuf 3.0.0-alpha.2. Do not edit
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

//! Generated file from `postgres.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_3_0_0_ALPHA_2;

#[derive(PartialEq,Clone,Default)]
pub struct Postgres {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a Postgres {
    fn default() -> &'a Postgres {
        <Postgres as ::protobuf::Message>::default_instance()
    }
}

impl Postgres {
    pub fn new() -> Postgres {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::new();
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<Postgres>(
            "Postgres",
            0,
            fields,
        )
    }
}

impl ::protobuf::Message for Postgres {
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

    fn new() -> Postgres {
        Postgres::new()
    }

    fn descriptor_static() -> ::protobuf::reflect::MessageDescriptor {
        ::protobuf::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 0)
    }

    fn default_instance() -> &'static Postgres {
        static instance: Postgres = Postgres {
            unknown_fields: ::protobuf::UnknownFields::new(),
            cached_size: ::protobuf::rt::CachedSize::new(),
        };
        &instance
    }
}

impl ::protobuf::Clear for Postgres {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Postgres {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Postgres {
    type RuntimeType = ::protobuf::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

#[derive(PartialEq,Clone,Default)]
pub struct PostgresService {
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::rt::CachedSize,
}

impl<'a> ::std::default::Default for &'a PostgresService {
    fn default() -> &'a PostgresService {
        <PostgresService as ::protobuf::Message>::default_instance()
    }
}

impl PostgresService {
    pub fn new() -> PostgresService {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::new();
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<PostgresService>(
            "PostgresService",
            1,
            fields,
        )
    }
}

impl ::protobuf::Message for PostgresService {
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

    fn new() -> PostgresService {
        PostgresService::new()
    }

    fn descriptor_static() -> ::protobuf::reflect::MessageDescriptor {
        ::protobuf::reflect::MessageDescriptor::new_generated_2(file_descriptor(), 1)
    }

    fn default_instance() -> &'static PostgresService {
        static instance: PostgresService = PostgresService {
            unknown_fields: ::protobuf::UnknownFields::new(),
            cached_size: ::protobuf::rt::CachedSize::new(),
        };
        &instance
    }
}

impl ::protobuf::Clear for PostgresService {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for PostgresService {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for PostgresService {
    type RuntimeType = ::protobuf::reflect::runtime_types::RuntimeTypeMessage<Self>;
}

/// Extension fields
pub mod exts {

    pub const postgres: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::MethodOptions, ::protobuf::reflect::types::ProtobufTypeMessage<super::Postgres>> = ::protobuf::ext::ExtFieldOptional { field_number: 50021, phantom: ::std::marker::PhantomData };

    pub const postgres_service: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::ServiceOptions, ::protobuf::reflect::types::ProtobufTypeMessage<super::PostgresService>> = ::protobuf::ext::ExtFieldOptional { field_number: 50022, phantom: ::std::marker::PhantomData };
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0epostgres.proto\x12\rpandit.format\x1a\x20google/protobuf/descripto\
    r.proto\"\n\n\x08Postgres\"\x11\n\x0fPostgresService:U\n\x08postgres\x18\
    \xe5\x86\x03\x20\x01(\x0b2\x17.pandit.format.Postgres\x12\x1e.google.pro\
    tobuf.MethodOptionsR\x08postgres:l\n\x10postgres_service\x18\xe6\x86\x03\
    \x20\x01(\x0b2\x1e.pandit.format.PostgresService\x12\x1f.google.protobuf\
    .ServiceOptionsR\x0fpostgresServiceJ\xf8\x01\n\x06\x12\x04\0\0\x10\x01\n\
    \x08\n\x01\x0c\x12\x03\0\0\x12\n\t\n\x02\x03\0\x12\x03\x01\0*\n\x08\n\
    \x01\x02\x12\x03\x03\0\x16\n\t\n\x02\x04\0\x12\x03\x05\0\x13\n\n\n\x03\
    \x04\0\x01\x12\x03\x05\x08\x10\n\t\n\x02\x04\x01\x12\x03\x07\0\x1a\n\n\n\
    \x03\x04\x01\x01\x12\x03\x07\x08\x17\n\x20\n\x01\x07\x12\x03\x0b\0C2\x16\
    \x20enum\x20PostgresType\x20{}\n\n\t\n\x02\x07\0\x12\x03\x0b'A\n\n\n\x03\
    \x07\0\x02\x12\x03\x0b\x07$\n\n\n\x03\x07\0\x06\x12\x03\x0b'/\n\n\n\x03\
    \x07\0\x01\x12\x03\x0b08\n\n\n\x03\x07\0\x03\x12\x03\x0b;@\n\t\n\x01\x07\
    \x12\x04\x0e\0\x10\x01\n\t\n\x02\x07\x01\x12\x03\x0f\x02+\n\n\n\x03\x07\
    \x01\x02\x12\x03\x0e\x07%\n\n\n\x03\x07\x01\x06\x12\x03\x0f\x02\x11\n\n\
    \n\x03\x07\x01\x01\x12\x03\x0f\x12\"\n\n\n\x03\x07\x01\x03\x12\x03\x0f%*\
    b\x06proto3\
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
        messages.push(Postgres::generated_message_descriptor_data());
        messages.push(PostgresService::generated_message_descriptor_data());
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
