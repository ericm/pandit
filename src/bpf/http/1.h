#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "../bpf_helpers/builtins.h"
#include "../bpf_helpers/str.h"
#include "../utils/hash.h"

#define ascii_offset 48

#define http_offset_version 5
#define http_offset_code 9

static __u8 HTTP[] = "HTTP";
static __u8 kv[2] = ":";
static __u8 el[3] = "\r\n";

PROG(PDT_HTTP_1)
(struct xdp_md *ctx)
{
}