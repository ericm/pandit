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
//! Generated file from `hander.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

/// Extension fields
pub mod exts {

    pub const json: ::protobuf::ext::ExtFieldOptional<::protobuf::descriptor::MessageOptions, ::protobuf::types::ProtobufTypeBool> = ::protobuf::ext::ExtFieldOptional { field_number: 50040, phantom: ::std::marker::PhantomData };
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0chander.proto\x12\x0epandit.handler\x1a\x20google/protobuf/descript\
    or.proto:5\n\x04json\x18\xf8\x86\x03\x20\x01(\x08\x12\x1f.google.protobu\
    f.MessageOptionsR\x04jsonJl\n\x06\x12\x04\0\0\x05<\n\x08\n\x01\x0c\x12\
    \x03\0\0\x12\n\t\n\x02\x03\0\x12\x03\x01\0*\n\x08\n\x01\x02\x12\x03\x03\
    \0\x17\n\x08\n\x01\x07\x12\x03\x05\0<\n\t\n\x02\x07\0\x12\x03\x05(:\n\n\
    \n\x03\x07\0\x02\x12\x03\x05\x07%\n\n\n\x03\x07\0\x05\x12\x03\x05(,\n\n\
    \n\x03\x07\0\x01\x12\x03\x05-1\n\n\n\x03\x07\0\x03\x12\x03\x0549b\x06pro\
    to3\
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
