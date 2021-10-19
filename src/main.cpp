#include <iostream>
#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <xdp_parser.skel.h>

static int libbpf_print_fn(enum libbpf_print_level level, const char *format, va_list args)
{
    return vfprintf(stderr, format, args);
}

int main() {
    struct xdp_parser_bpf *skel;
    int err;
    int lookups_fd;

    libbpf_set_print(libbpf_print_fn);

    skel = xdp_parser_bpf__open_and_load();
    if (!skel) {
        std::cerr << "Failed to open and load BPF skeleton" << std::endl;
        return 1;
    }

    std::cout << "Successfully started! Please run `sudo cat /sys/kernel/debug/tracing/trace_pipe`" << std::endl;

    err = xdp_parser_bpf__attach(skel);
    if (err) {
        std::cerr << "Failed to attach BPF skeleton" << std::endl;
    }

    lookups_fd = bpf_map__fd(skel->maps.lookups);

    char *key, *c_pack;

    bpf_map_get_next_key(lookups_fd, key, &key);
    bpf_map_lookup_elem(lookups_fd, key, c_pack);

    std::string pack(c_pack);

    return 0;
}
