#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "../bpf_helpers/builtins.h"
#include "../bpf_helpers/str.h"
#include "../utils/hash.h"

#define ascii_offset 48
#define MTU 1500

#define http_offset_version 5
#define http_offset_code 9

typedef struct pdt_http1_req_hdr_t
{
    __u8 maj_version;
    __u8 min_version;
    __u8 code;
    pdt_hash_t *hdr_map;
} pdt_http1_req_hdr_t;

static __u8 HTTP[] = "HTTP";
static __u8 kv[2] = ":";
static __u8 el[3] = "\r\n";

pdt_hash_t pdt_http1_req_hdr_map SEC(".maps");

static __always_inline __maybe_unused int
pdt_parse_http1_req_hdr(pdt_http1_req_hdr_t *hdr, __u8 *buf, __u16 len)
{
    char h_buf[3];
    char *sb, *sb_sep;
    int lower, i;

    if (len > MTU)
        return 0;

    if (__bpf_memcmp(&HTTP, buf, sizeof(HTTP) - 1))
        return 0;

    __bpf_memcpy_builtin(&h_buf, buf + http_offset_version, sizeof(h_buf));
    hdr->maj_version = h_buf[0] - ascii_offset;
    hdr->min_version = h_buf[2] - ascii_offset;
    bpf_printk("version: %d.%d", hdr->maj_version, hdr->min_version);

    __bpf_memcpy_builtin(&h_buf, buf + http_offset_code, sizeof(h_buf));
    hdr->code = (h_buf[0] * 100 + h_buf[1] * 10 + h_buf[2]) - (ascii_offset * 3);

    bpf_printk("len %d", len);

    pdt_buff_t pdt_buff = {.buf = buf,
                           .size = len,
                           .offset = http_offset_code + sizeof(h_buf)};
    pdt_buff_t kv_sep = {.buf = kv, .size = 1, .offset = 0};
    pdt_buff_t el_sep = {.buf = el, .size = 2, .offset = 0};
    pdt_hash_populate(&pdt_http1_req_hdr_map, &pdt_buff, &kv_sep, &el_sep);

    hdr->hdr_map = &pdt_http1_req_hdr_map;
    return 1;
}
