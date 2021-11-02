#include "vmlinux.h"
#include "parsing_helpers.h"
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

static int parse_header(char *data, const char *data_end, size_t offset, char ***hdr) {
    if (data+offset >= data_end)
        return XDP_PASS;

    char *l_hdr = data+offset;
    **hdr = l_hdr;
    bpf_printk("HDR = %p", hdr);
    return -1;
}

static int from_data(struct xdp_md *ctx, char **payload, const u32 *ip, char **data, char **data_end) {
    struct iphdr *ip_hdr;
    char *l_data, *l_data_end;
    int ret;

    if (*data<*data_end)
        return -1;

    l_data = (char *)(long) ctx->data;
    l_data_end = (char *)(long)ctx->data_end;
    *data = l_data;
    *data_end = l_data_end;

//    parse_header(l_data, l_data_end, sizeof(struct ethhdr), &ip_hdr);
    ret = parse_header(l_data, l_data_end, sizeof(struct ethhdr) + sizeof(struct iphdr) + sizeof(struct tcphdr), &payload);
    if (ret > -1) {
        return ret;
    }
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
        bpf_printk("Packet passed: %s", payload);
        bpf_map_update_elem(&lookups, &ip, &payload, BPF_ANY);
    }
    return -1;
}

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx) {
    bpf_printk("Packet received");

    void *data_end = (void *)(unsigned long long)ctx->data_end;
    void *data = (void *)(unsigned long long)ctx->data;

    struct hdr_cursor nh;
    struct ethhdr *eth;
    int eth_type;
    int ip_type;
    int tcp_type;
    struct iphdr *iphdr;
    struct ipv6hdr *ipv6hdr;
    struct tcphdr *tcphdr;

    nh.pos = data;

    eth_type = parse_ethhdr(&nh, data_end, &eth);
    if (eth_type == bpf_htons(ETH_P_IP)) {
        ip_type = parse_iphdr(&nh, data_end, &iphdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
    }
    else if (eth_type == bpf_htons(ETH_P_IPV6)) {
        ip_type = parse_ip6hdr(&nh, data_end, &ipv6hdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
    } else {
        return XDP_PASS;
    }

    tcp_type = parse_tcphdr(&nh, data_end, &tcphdr);
    if ((void *)(tcphdr + 1) > data_end) {
        return XDP_PASS;
    }

    switch (tcphdr->dest) {
        case bpf_htons(8000):
            bpf_printk("Right Port");
    }

    return XDP_PASS;
}
