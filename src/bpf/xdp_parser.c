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
    int pld_len, i, body_loc = 0;
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

    // https://github.com/xdp-project/xdp-tools/blob/892e23248b0275f2d9defaddc8350469febca486/headers/linux/bpf.h#L2563
    // pld_len = iphdr->tot_len - hdrlen;
    for (i = 0; i + 1 < (data_end - data) && i < 150; i++)
    {
        if (data + hdrlen + i + 5 < data_end && __bpf_memcmp(data + i + hdrlen, hdr_split, 4) == 0)
        {
            body_loc = i + 4;
            break;
        }
        // bpf_printk("%d", *(data + i));
    }
    bpf_printk("Body loc: %d", body_loc);
    return XDP_PASS;
}
