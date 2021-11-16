#include "../vmlinux.h"
#include "../utils/hash.h"
#include <bpf/bpf_helpers.h>

typedef struct pdt_http_req_hdr_t
{
    __u8 maj_version;
    __u8 min_version;
    __u8 code;
} pdt_http_req_hdr_t;

typedef enum pdt_req_hdrs
{
    PDT_TYPE_CONTENT_LENGTH,
} pdt_req_hdrs_t;

pdt_hash_el_t pdt_http_req_hdr_content_length SEC(".maps"),
    pdt_http_req_hdr_host SEC(".maps");

struct pdt_http_req_hdr_hash
{
    __type(key, pdt_req_hdrs_t);
    pdt_hash_base
} pdt_http_req_hdr_hash SEC(".maps") = {
    .values = {
        [PDT_TYPE_CONTENT_LENGTH] = &pdt_http_req_hdr_content_length,
    },
};