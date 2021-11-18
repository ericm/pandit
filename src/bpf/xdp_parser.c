#include "vmlinux.h"
#include "xdp/parsing_helpers.h"
#include "xdp/context_helpers.h"
#include "bpf_helpers/builtins.h"
#include <bpf/bpf_helpers.h>

// typedef struct
// {
//     __uint(type, BPF_MAP_TYPE_STACK);
//     __uint(max_entries, 8192);
//     __type(value, struct );
// } pdt_stack_map_t;

char LICENSE[] SEC("license") = "Dual BSD/GPL";

#define static_offset4 \
    sizeof(struct ethhdr) + sizeof(struct iphdr) + sizeof(struct tcphdr)

#define static_offset6 \
    sizeof(struct ethhdr) + sizeof(struct ipv6hdr) + sizeof(struct tcphdr)

#define static_mtu4 1500
#define static_read4 288

static __u8 buf[static_mtu4];

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx)
{
    bpf_printk("Packet received");

    __u8 *data_end = (__u8 *)(unsigned long long)ctx->data_end;
    __u8 *data = (__u8 *)(unsigned long long)ctx->data;

    struct hdr_cursor cursor;
    struct ethhdr *eth;
    int eth_type;
    int ip_type;
    int hdrlen;
    int pld_len, i;
    struct iphdr *iphdr;
    struct ipv6hdr *ipv6hdr;
    struct tcphdr *tcphdr;
    char hdr_split[4] = "\r\n\r\n";

    cursor.pos = data;
    cursor.end = data_end;

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
    cursor.pos += tcphdr->doff * 4;
    if (tcphdr->dest != bpf_htons(8000) && tcphdr->source != bpf_htons(8000))
    {
        return XDP_PASS;
    }
    bpf_printk("Right Port");

    if (eth_type == bpf_htons(ETH_P_IP))
    {
    }
    else
    {
        bpf_printk("v6");
        xdp_load_bytes(ctx, static_offset6, buf, static_offset6);
    }
    // https://github.com/xdp-project/xdp-tools/blob/892e23248b0275f2d9defaddc8350469febca486/headers/linux/bpf.h#L2563
    pld_len = iphdr->tot_len - hdrlen;
    for (i = 0; i + static_read4 + 1 < pld_len && i + static_read4 + 1 < (data_end - data) && i + static_read4 < sizeof(buf); i += static_read4)
    {
        xdp_load_bytes(ctx, hdrlen + i, &buf[i], static_read4);
    }
    for (i = 0; i < sizeof(buf) - 4; i++)
    {
        if (__bpf_memcmp(&buf[i], hdr_split, 4))
        {
            bpf_printk("Split");
            break;
        }
    }
    // if (i < pld_len) {
    //     xdp_load_bytes(ctx, hdrlen + i, buf + i, static_read4);
    // }

    return XDP_PASS;
}
