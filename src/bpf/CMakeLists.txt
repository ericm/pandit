cmake_minimum_required(VERSION 3.16)

project(pandit-bpf)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../../tools/cmake)
set(BPFOBJECT_BPFTOOL_EXE ${CMAKE_CURRENT_SOURCE_DIR}/../../tools/bpftool)
set(BPFOBJECT_VMLINUX_H ${CMAKE_CURRENT_SOURCE_DIR}/../../vmlinux/vmlinux.h)
find_package(BpfObject REQUIRED)
file(GLOB apps *.bpf.c)

# Create an executable for each application
file(GLOB apps *.bpf.c)
foreach(app ${apps})
    get_filename_component(app_stem ${app} NAME_WE)

    # Build object skeleton and depend skeleton on libbpf build
    bpf_object(${app_stem} ${app_stem}.bpf.c)
    add_dependencies(${app_stem}_skel libbpf-build)
endforeach()


