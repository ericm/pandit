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
#define ascii_offset 48

typedef struct
{
    __u32 body_loc;
} pdt_http_resp_t;

struct
{
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 8192);
    __type(key, __u64);
    __type(value, pdt_http_resp_t);
} pdt_ip_hash_map SEC(".maps");

typedef struct
{
    __u32 value_ptr;
    __u32 len;
    union
    {
        __u8 token;
        enum
        {
            PDT_TOKEN_KEY,
            PDT_TOKEN_VALUE,
            PDT_TOKEN_STRING,
            PDT_TOKEN_NUMBER,
            PDT_TOKEN_BOOLEAN
        } type;
    } scope;
} pdt_parse_sym_t;

struct pdt_parse_map_t
{
    __uint(type, BPF_MAP_TYPE_STACK);
    __uint(max_entries, 32);
    __uint(map_flags, 0);
    __uint(key_size, 0);
    __uint(value_size, sizeof(pdt_parse_sym_t));
} pdt_parse_map SEC(".maps");

static __u8 buf[static_mtu4];
static const __u8 HTTP[] = "HTTP";
static const __u8 HDR_SPLIT[] = "Content-Length";

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx)
{

    __u8 *data_end = (__u8 *)(unsigned long long)ctx->data_end;
    __u8 *data = (__u8 *)(unsigned long long)ctx->data;
    bpf_printk("Packet received %d", data_end - data);

    struct hdr_cursor cursor;
    pdt_http_resp_t *resp;
    struct ethhdr *eth;
    int eth_type;
    int ip_type;
    int hdrlen;
    int pld_len, i, body_loc = 0;
    struct iphdr *iphdr;
    struct ipv6hdr *ipv6hdr;
    struct tcphdr *tcphdr;
    __u64 key;
    __u8 maj_ver, min_ver, code;
    pdt_parse_sym_t sym = {.len = 0, .value_ptr = 0};

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

    bpf_printk("%u %u %u", tcphdr->seq, tcphdr->ack_seq, tcphdr->ack);
    bpf_printk("%s", (data + hdrlen));
    if (tcphdr->dest != bpf_htons(8000) && tcphdr->source != bpf_htons(8000))
    {
        return XDP_PASS;
    }
    bpf_printk("Right Port");
    key = ((__u64)iphdr->daddr << 32) | tcphdr->ack_seq;
    resp = bpf_map_lookup_elem(&pdt_ip_hash_map, &key);
    // replace solid tokens with maps
    if (resp)
    {
        bpf_printk("Found");
        for (i = 0; i < static_mtu4 / 2 - hdrlen; i++)
        {
            if (data + hdrlen + i + 1 > data_end)
            {
                return XDP_PASS;
            }
            switch (*(data + hdrlen + i))
            {
            case '{':
                sym.scope.token = '{';
                bpf_map_push_elem(&pdt_parse_map, &sym, 0);
                bpf_printk("Found {");
                break;
            case '}':
                bpf_map_pop_elem(&pdt_parse_map, &sym);
                bpf_printk("Found }");
                break;
            case '"':
                bpf_printk("Found \"");
                break;
            case ':':
                sym.value_ptr = 0;
                sym.len = 0;
                sym.scope.type = PDT_TOKEN_VALUE;
                bpf_map_push_elem(&pdt_parse_map, &sym, 0);
                bpf_printk("Found :");
                break;
            case ' ':
            case '\t':
                bpf_map_pop_elem(&pdt_parse_map, &sym);
                if ((sym.scope.type == PDT_TOKEN_STRING ||
                     sym.scope.type == PDT_TOKEN_KEY))
                {
                    sym.len++;
                }
                break;
            default:
                bpf_map_pop_elem(&pdt_parse_map, &sym);
                if (sym.value_ptr == 0)
                {
                    // sym.value_ptr = data + hdrlen + i;
                    sym.len = 1;
                    break;
                }
                sym.len++;
                bpf_map_push_elem(&pdt_parse_map, &sym, 0);
                break;
            }
        }
        return XDP_PASS;
    }

    if (data + hdrlen + sizeof(HTTP) + 7 > data_end)
    {
        return XDP_PASS;
    }
    if (__bpf_memcmp(&HTTP, data + hdrlen, sizeof(HTTP) - 1))
        return XDP_PASS;
    maj_ver = *(data + hdrlen + sizeof(HTTP)) - ascii_offset, min_ver = *(data + hdrlen + sizeof(HTTP) + 2) - ascii_offset;
    code = (*(data + hdrlen + sizeof(HTTP) + 4) - ascii_offset) * 100;
    code += (*(data + hdrlen + sizeof(HTTP) + 5) - ascii_offset) * 10;
    code += (*(data + hdrlen + sizeof(HTTP) + 6) - ascii_offset);
    bpf_printk("ver %d.%d status %d", maj_ver, min_ver, code);

    // https://github.com/xdp-project/xdp-tools/blob/892e23248b0275f2d9defaddc8350469febca486/headers/linux/bpf.h#L2563
    // pld_len = iphdr->tot_len - hdrlen;
    for (i = 0; i + 1 < (data_end - data) && i < 200; i++)
    {
        if (data + hdrlen + i + sizeof(HDR_SPLIT) > data_end)
        {
            break;
        }
        if (__bpf_memcmp(&HDR_SPLIT, data + i + hdrlen, sizeof(HDR_SPLIT) - 1) == 0)
        {
            bpf_printk("Found split");
            pdt_http_resp_t n_resp = {};
            bpf_map_update_elem(&pdt_ip_hash_map, &key, &n_resp, BPF_ANY);
            body_loc = i + 4;
            break;
        }
    }
    bpf_printk("Body loc: %d %d", body_loc, data_end - data);
    return XDP_PASS;
}
