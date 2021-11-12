#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "../bpf_helpers/builtins.h"

#define ascii_offset 48;

static __u8 HTTP[] = "HTTP";

struct http1_1_req_hdr_t {
    __u8 maj_version;
    __u8 min_version;
    __u8 code;
};

static __always_inline __maybe_unused int
parse_http1_1_req_hdr(struct http1_1_req_hdr_t *hdr, const __u8 *buf, int len) {
    char h_buf[3];
    int lower, i;
    char *sb, *sb_sep;


    if(__bpf_memcmp(&HTTP, buf, sizeof(HTTP)-1))
        return 0;

    __bpf_memcpy_builtin(&h_buf, buf+5, sizeof(h_buf));

    hdr->maj_version = h_buf[0]-ascii_offset;
    hdr->min_version = h_buf[2]-ascii_offset;
    bpf_printk("version: %d.%d", hdr->maj_version, hdr->min_version);

    __bpf_memcpy_builtin(&h_buf, buf+9, sizeof(h_buf));
    hdr->code = (h_buf[0]*100 + h_buf[1]*10 + h_buf[2]) - ascii_offset - ascii_offset - ascii_offset;

    bpf_printk("len %d", len);
    
    // for (i = 13; i < len; i++) {
    //     sb = strchr(sb, 13) + 1;
    //     sb_sep = strchr(sb, ':');
    // }

    return 1;
};