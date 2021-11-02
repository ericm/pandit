#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_tracing.h>

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

static void parse_header(char *data, const char *data_end, size_t offset, char ***hdr) {
    if (data+offset > data_end)
        return;

    char *l_hdr = data+offset;
    **hdr = l_hdr;
    bpf_printk("HDR = %p", hdr);
}

static int from_data(struct xdp_md *ctx, char **payload, const u32 *ip, char **data, char **data_end) {
    struct iphdr *ip_hdr;
    char *l_data, *l_data_end;

    if (*data<*data_end)
        return -1;

    l_data = (char *)(long) ctx->data;
    l_data_end = (char *)(long)ctx->data_end;
    data = &l_data;
    data_end = &l_data_end;

//    parse_header(l_data, l_data_end, sizeof(struct ethhdr), &ip_hdr);
    parse_header(l_data, l_data_end, sizeof(struct ethhdr) + sizeof(struct iphdr) + sizeof(struct tcphdr), &payload);
//    ip = &ip_hdr->daddr;

    return -1;
}

static int parse_payload(u32 *ip, char *data, char *payload) {
    struct lookup *lookup = bpf_map_lookup_elem(&lookups, &ip);

    if (!lookup && payload) {
        for (int i = 0; i < sizeof(HTTP); i++) {
            if (*(payload+i) != HTTP[i]) {
                bpf_printk("Error parsing %s", *payload);
                return XDP_PASS;
            }
        }
        bpf_map_update_elem(&lookups, &ip, &payload, BPF_ANY);
    }
    return -1;
}

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx) {
    bpf_printk("Packet received");
    char *data, *data_end, *payload;
//    https://github.com/xdp-project/xdp-tutorial/blob/master/packet02-rewriting/xdp_prog_kern.c
    u32 ip;
    int ret;

    ret = from_data(ctx, &payload, &ip, &data, &data_end);
    if (ret > -1)
        return ret;

    bpf_printk("Payload = %p", payload);
    if (data_end-payload < sizeof(HTTP))
        return XDP_PASS;
    ret = parse_payload(&ip, data, payload);
    if (ret > -1)
        return ret;

    return XDP_PASS;
}
