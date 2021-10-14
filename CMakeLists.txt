cmake_minimum_required(VERSION 3.21)
project(pandit)

set(CMAKE_CXX_STANDARD 20)

add_executable(pandit main.cpp ebpf-src/xdp_parser.bpf.c)
