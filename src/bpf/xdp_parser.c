#include "vmlinux.h"
#include "xdp/parsing_helpers.h"
#include "xdp/context_helpers.h"
#include "bpf_helpers/builtins.h"
#include <bpf/bpf_helpers.h>
#include "http/1.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";

#define static_offset4 \
    sizeof(struct ethhdr) + sizeof(struct iphdr) + sizeof(struct tcphdr)

#define static_offset6 \
    sizeof(struct ethhdr) + sizeof(struct ipv6hdr) + sizeof(struct tcphdr)

static __u8 buf[static_offset4 + 1500];

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx)
{
    bpf_printk("Packet received");

    void *data_end = (void *)(unsigned long long)ctx->data_end;
    void *data = (void *)(unsigned long long)ctx->data;

    struct hdr_cursor cursor;
    struct ethhdr *eth;
    int eth_type;
    int ip_type;
    int hdrlen;
    struct iphdr *iphdr;
    struct ipv6hdr *ipv6hdr;
    struct tcphdr *tcphdr;

    pdt_http1_req_hdr_t req_hdr = {};

    cursor.pos = data;
    hdrlen = sizeof(struct ethhdr);

    eth_type = parse_ethhdr(&cursor, data_end, &eth);
    if (eth_type == bpf_htons(ETH_P_IP))
    {
        ip_type = parse_iphdr(&cursor, data_end, &iphdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
        hdrlen += sizeof(struct iphdr);
    }
    else if (eth_type == bpf_htons(ETH_P_IPV6))
    {
        ip_type = parse_ip6hdr(&cursor, data_end, &ipv6hdr);
        if (ip_type != IPPROTO_TCP)
            return XDP_PASS;
        hdrlen += sizeof(struct ipv6hdr);
    }
    else
    {
        return XDP_PASS;
    }

    parse_tcphdr(&cursor, data_end, &tcphdr);
    if ((void *)(tcphdr + 1) > data_end)
    {
        return XDP_PASS;
    }

    hdrlen += tcphdr->doff * 4;

    if (tcphdr->dest != bpf_htons(8000) && tcphdr->source != bpf_htons(8000))
    {
        return XDP_PASS;
    }
    bpf_printk("Right Port");

    if (eth_type == bpf_htons(ETH_P_IP))
    {
        // bpf_printk("v4 %x %d %d", tcphdr->window, hdrlen, tcphdr->doff);
        xdp_load_bytes(ctx, hdrlen, buf, static_offset4);
    }
    else
    {
        bpf_printk("v6");
        xdp_load_bytes(ctx, static_offset6, buf, static_offset6);
    }
    int i;
    for (i = 0; i < static_offset4; i++)
    {
        bpf_printk("= %d", buf[i]);
    }

    pdt_parse_http1_req_hdr(&req_hdr, buf, sizeof(buf));
    return XDP_PASS;
}
