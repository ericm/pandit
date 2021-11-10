#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "../bpf_helpers/builtins.h"

#define ascii_offset 48;

struct http1_1_req_hdr_t {
    __u8 version;
};

static __always_inline __maybe_unused int
parse_http1_1_req_hdr(struct http1_1_req_hdr_t *hdr, const __u8 *buf, int len) {
    char version[3];
    int lower;

    bpf_printk("bf %d", version[0]);
    __bpf_memcpy_builtin(&version, buf+4, sizeof(version));
    bpf_printk("af %d", version[0]);
    hdr->version = version[0]-ascii_offset;

    lower = version[2]-ascii_offset;
    lower <<= sizeof(int)/2;
    hdr->version = hdr->version & lower;
    return 1;
};