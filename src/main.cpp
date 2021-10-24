#include <iostream>
#include <thread>
#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <xdp_parser.skel.h>

static int libbpf_print_fn(enum libbpf_print_level level, const char *format, va_list args)
{
    return vfprintf(stderr, format, args);
}

int main() {
    struct xdp_parser_bpf *skeleton;
    int err;
    int lookups_fd, prog_fd;
    int if_index;

    if_index = 1;

    libbpf_set_print(libbpf_print_fn);

    skeleton = xdp_parser_bpf__open_and_load();
    if (!skeleton) {
        std::cerr << "Failed to open and load BPF skeleton" << std::endl;
        return 1;
    }

    err = xdp_parser_bpf__attach(skeleton);
    if (err) {
        std::cerr << "Failed to attach BPF skeleton" << std::endl;
        return 1;
    }

    prog_fd = bpf_object__btf_fd(skeleton->obj);
    err = bpf_set_link_xdp_fd(if_index, prog_fd, 0);
    if (err) {
        std::cerr << "Failed to attach XDP to interface: " << if_index << std::endl;
        return 1;
    }

    lookups_fd = bpf_map__fd(skeleton->maps.lookups);

    char *key, *c_pack;
    while (true) {
        std::cout << "Successfully started! Please run `sudo cat /sys/kernel/debug/tracing/trace_pipe`" << std::endl;
        bpf_map_get_next_key(lookups_fd, key, &key);
        bpf_map_lookup_elem(lookups_fd, key, &c_pack);

        if (!c_pack) {
            sleep(3);
            continue;
        }

        std::string pack(c_pack);

        std::cout << pack << std::endl;
        return 0;
    }
}
