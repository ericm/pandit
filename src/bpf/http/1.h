#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "../bpf_helpers/builtins.h"
#include "../bpf_helpers/str.h"
#include "../utils/hash.h"

#define ascii_offset 48

#define http_offset_version 5
#define http_offset_code 9

typedef struct pdt_http1_req_hdr_t
{
    __u8 maj_version;
    __u8 min_version;
    __u8 code;
    pdt_hash_t *hdr_map;
} pdt_http1_req_hdr_t;

static __always_inline __maybe_unused int
pdt_parse_http1_req_hdr(pdt_http1_req_hdr_t **hdr, const __u8 *buf, size_t len);
