package(default_visibility = ["//lib:__subpackages__"])

load("//build:ebpf.bzl", "cc_ebpf")

cc_ebpf(
    name = "xdp_parser",
    srcs = ["xdp_parser.bpf.c"],
    hdrs = ["vmlinux.h"],
    copts = ["-g"],
    deps = ["@libbpf"],
)
