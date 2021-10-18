load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_foreign_cc",
    sha256 = "b30d354592980848dd1ecc120de3de34a7c574985e85b3ae2bcecd174fd007b8",
    strip_prefix = "rules_foreign_cc-c3d5405cbc570257e7c9f75f902fab42241e6a53",
    url = "https://github.com/bazelbuild/rules_foreign_cc/archive/c3d5405.zip",
)

load("@rules_foreign_cc//:workspace_definitions.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies()

http_archive(
    name = "elfutils",
    build_file = "//third_party:BUILD.elf",
    strip_prefix = "elfutils-0.185",
    url = "https://sourceware.org/elfutils/ftp/0.185/elfutils-0.185.tar.bz2",
)

http_archive(
    name = "libbpf",
    build_file = "//third_party:BUILD.bpf",
    sha256 = "c89ca0958674e1efcfec5f92554b4596a592ee637d18657302382f7e57ddbea6",
    strip_prefix = "libbpf-0.0.4",
    url = "https://github.com/libbpf/libbpf/archive/v0.0.4.zip",
)

http_archive(
    name = "gtest",
    sha256 = "927827c183d01734cc5cfef85e0ff3f5a92ffe6188e0d18e909c5efebf28a0c7",
    strip_prefix = "googletest-release-1.8.1",
    url = "https://github.com/google/googletest/archive/release-1.8.1.zip",
)

#
# Download the longterm stable kernel version in 4.1x series at the time of
# writing this;
# NOTE: This is a moving target
http_archive(
    name = "linuxsrc",
    build_file = "//third_party:BUILD.install_linux_hdr",
    sha256 = "324d8967fbda539731a71a1a2fd469c85eda0a6459c8b172e84a8d20cda410b3",
    strip_prefix = "linux-5.13",
    url = "https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.13.tar.gz",
)

#############################################################################
# All rules below are to configure the bazel remote build environment, and bring
# in clang-8 based toolchains on your system automatically.
#
# More details are here: https://github.com/bazelbuild/bazel-toolchains/

# Change master to the git tag you want.
BAZEL_TOOLCHAIN_TAG = "0.6.3"

BAZEL_TOOLCHAIN_SHA = "da607faed78c4cb5a5637ef74a36fdd2286f85ca5192222c4664efec2d529bb8"

http_archive(
    name = "com_grail_bazel_toolchain",
    canonical_id = BAZEL_TOOLCHAIN_TAG,
    sha256 = BAZEL_TOOLCHAIN_SHA,
    strip_prefix = "bazel-toolchain-{tag}".format(tag = BAZEL_TOOLCHAIN_TAG),
    url = "https://github.com/grailbio/bazel-toolchain/archive/{tag}.tar.gz".format(tag = BAZEL_TOOLCHAIN_TAG),
)

load("@com_grail_bazel_toolchain//toolchain:rules.bzl", "llvm_toolchain")

llvm_toolchain(
    name = "llvm_toolchain",
    #distribution = "clang+llvm-8.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz",
    llvm_version = "12.0.0",
)

load("@llvm_toolchain//:toolchains.bzl", "llvm_register_toolchains")

llvm_register_toolchains()

http_archive(
    name = "bazel_toolchains",
    sha256 = "1adf5db506a7e3c465a26988514cfc3971af6d5b3c2218925cd6e71ee443fc3f",
    strip_prefix = "bazel-toolchains-4.0.0",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-toolchains/releases/download/4.0.0/bazel-toolchains-4.0.0.tar.gz",
        "https://github.com/bazelbuild/bazel-toolchains/releases/download/4.0.0/bazel-toolchains-4.0.0.tar.gz",
    ],
)

load("@bazel_toolchains//rules:rbe_repo.bzl", "rbe_autoconfig")

# Creates a default toolchain config for RBE.
# Use this as is if you are using the rbe_ubuntu16_04 container,
# otherwise refer to RBE docs.
rbe_autoconfig(name = "rbe_default")
