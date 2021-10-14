#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/if_ether.h>
#include <linux/tcp.h>
#include <linux/ip.h>
#include <bits/stdint-uintn.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL";

const char HTTP[] = "HTTP/1.1";

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 8192);
    __type(key, uint32_t);
    __type(value, struct lookup);
} s SEC(".maps");

SEC("xdp")
int handle_egress_packet(struct xdp_md *ctx) {
    char *data = (char *)(long)ctx->data;
    char *data_end = (char *)(long)ctx->data_end;

    char *payload = data + sizeof(struct ethhdr)+sizeof(struct iphdr)+sizeof(struct tcphdr);
    struct iphdr *ip = (struct iphdr *)data+sizeof(struct ethhdr);
    struct tcphdr *tcp = (struct tcphdr *)data+sizeof(struct ethhdr)+sizeof(struct iphdr);

    uint64_t tuple = (ip->daddr << 16) + tcp->source;

    if ((data_end-data) < sizeof(HTTP))  {
        return XDP_PASS;
    }

    for (int i = 0; i < sizeof(HTTP); i++) {
        if (*(data+i) != HTTP[i]) {
            return XDP_PASS;
        }
    }

    if ()

}