#include "vmlinux.h"
#include "parsing_helpers.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_tracing.h>
#include <xdp/xdp_helpers.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL";

const char HTTP[] = "HTTP/1.1";

struct tuple_t {
    uint32_t ip;
    uint16_t port;
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 8192);
    u32 *key;
    char *value;
} lookups SEC(".maps");

/*
 * parse_tcppld: parse and return the length of the tcp payload
 */
static __always_inline int parse_tcppld(struct hdr_cursor *cursor,
                                        void *data_end,
                                        char **tcppld)
{
    int len;
    char *h = cursor->pos;

    if ((void *)(h + 1) > data_end)
        return -1;

    len = (int)((unsigned long long)data_end - (unsigned long long)h);
    /* Sanity check packet field is valid */
    if(len < sizeof(*h))
        return -1;

    /* Variable-length TCP header, need to use byte-based arithmetic */
    if (cursor->pos + len > data_end)
        return -1;

    cursor->pos += len;
    *tcppld = h;

    return len;
}

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx) {
    bpf_printk("Packet received");

    void *data_end = (void *)(unsigned long long)ctx->data_end;
    void *data = (void *)(unsigned long long)ctx->data;

    struct hdr_cursor cursor;
    struct ethhdr *eth;
    int eth_type;
    int ip_type;
    int tcp_type;
    struct iphdr *iphdr;
    struct ipv6hdr *ipv6hdr;
    struct tcphdr *tcphdr;
    char *tcppld;

    cursor.pos = data;

    eth_type = parse_ethhdr(&cursor, data_end, &eth);
    if (eth_type == bpf_htons(ETH_P_IP)) {
        ip_type = parse_iphdr(&cursor, data_end, &iphdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
    }
    else if (eth_type == bpf_htons(ETH_P_IPV6)) {
        ip_type = parse_ip6hdr(&cursor, data_end, &ipv6hdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
    } else {
        return XDP_PASS;
    }

    tcp_type = parse_tcphdr(&cursor, data_end, &tcphdr);
    if ((void *)(tcphdr + 1) > data_end) {
        return XDP_PASS;
    }

    switch (tcphdr->dest) {
        case bpf_htons(8000):
            bpf_printk("Right Port");
            break;
        default:
            return XDP_PASS;
    }

    parse_tcppld(&cursor, data_end, &tcppld);

    bpf_printk("Packet: %s", tcppld);

    return XDP_PASS;
}
