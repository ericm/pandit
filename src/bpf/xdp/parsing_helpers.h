/* SPDX-License-Identifier: (GPL-2.0-or-later OR BSD-2-clause) */
/*
 * This file contains parsing functions that are used in the packetXX XDP
 * programs. The functions are marked as __always_inline, and fully defined in
 * this header file to be included in the BPF program.
 *
 * Each helper parses a packet header, including doing bounds checking, and
 * returns the type of its contents if successful, and -1 otherwise.
 *
 * For Ethernet and IP headers, the content type is the type of the payload
 * (h_proto for Ethernet, nexthdr for IPv6), for ICMP it is the ICMP type field.
 * All return values are in host byte order.
 *
 * The versions of the functions included here are slightly expanded versions of
 * the functions in the packet01 lesson. For instance, the Ethernet header
 * parsing has support for parsing VLAN tags.
 */

#ifndef __PARSING_HELPERS_H
#define __PARSING_HELPERS_H

#include "../vmlinux.h"
#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>

/* If __USE_KERNEL_IPV6_DEFS is 1 then the user has included the kernel
   network headers first and we should use those ABI-identical definitions
   instead of our own, otherwise 0.  */
#if !__USE_KERNEL_IPV6_DEFS
enum
{
    IPPROTO_HOPOPTS = 0, /* IPv6 Hop-by-Hop options.  */
#define IPPROTO_HOPOPTS IPPROTO_HOPOPTS
    IPPROTO_ROUTING = 43, /* IPv6 routing header.  */
#define IPPROTO_ROUTING IPPROTO_ROUTING
    IPPROTO_FRAGMENT = 44, /* IPv6 fragmentation header.  */
#define IPPROTO_FRAGMENT IPPROTO_FRAGMENT
    IPPROTO_ICMPV6 = 58, /* ICMPv6.  */
#define IPPROTO_ICMPV6 IPPROTO_ICMPV6
    IPPROTO_NONE = 59, /* IPv6 no next header.  */
#define IPPROTO_NONE IPPROTO_NONE
    IPPROTO_DSTOPTS = 60, /* IPv6 destination options.  */
#define IPPROTO_DSTOPTS IPPROTO_DSTOPTS
    IPPROTO_MH = 135 /* IPv6 mobility header.  */
#define IPPROTO_MH IPPROTO_MH
};
#endif /* !__USE_KERNEL_IPV6_DEFS */

/*
 *	These are the defined Ethernet Protocol ID's.
 */

#define ETH_P_LOOP 0x0060      /* Ethernet Loopback packet	*/
#define ETH_P_PUP 0x0200       /* Xerox PUP packet		*/
#define ETH_P_PUPAT 0x0201     /* Xerox PUP Addr Trans packet	*/
#define ETH_P_TSN 0x22F0       /* TSN (IEEE 1722) packet	*/
#define ETH_P_ERSPAN2 0x22EB   /* ERSPAN version 2 (type III)	*/
#define ETH_P_IP 0x0800        /* Internet Protocol packet	*/
#define ETH_P_X25 0x0805       /* CCITT X.25			*/
#define ETH_P_ARP 0x0806       /* Address Resolution packet	*/
#define ETH_P_BPQ 0x08FF       /* G8BPQ AX.25 Ethernet Packet	[ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_IEEEPUP 0x0a00   /* Xerox IEEE802.3 PUP packet */
#define ETH_P_IEEEPUPAT 0x0a01 /* Xerox IEEE802.3 PUP Addr Trans packet */
#define ETH_P_BATMAN 0x4305    /* B.A.T.M.A.N.-Advanced packet [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_DEC 0x6000       /* DEC Assigned proto           */
#define ETH_P_DNA_DL 0x6001    /* DEC DNA Dump/Load            */
#define ETH_P_DNA_RC 0x6002    /* DEC DNA Remote Console       */
#define ETH_P_DNA_RT 0x6003    /* DEC DNA Routing              */
#define ETH_P_LAT 0x6004       /* DEC LAT                      */
#define ETH_P_DIAG 0x6005      /* DEC Diagnostics              */
#define ETH_P_CUST 0x6006      /* DEC Customer use             */
#define ETH_P_SCA 0x6007       /* DEC Systems Comms Arch       */
#define ETH_P_TEB 0x6558       /* Trans Ether Bridging		*/
#define ETH_P_RARP 0x8035      /* Reverse Addr Res packet	*/
#define ETH_P_ATALK 0x809B     /* Appletalk DDP		*/
#define ETH_P_AARP 0x80F3      /* Appletalk AARP		*/
#define ETH_P_8021Q 0x8100     /* 802.1Q VLAN Extended Header  */
#define ETH_P_ERSPAN 0x88BE    /* ERSPAN type II		*/
#define ETH_P_IPX 0x8137       /* IPX over DIX			*/
#define ETH_P_IPV6 0x86DD      /* IPv6 over bluebook		*/
#define ETH_P_PAUSE 0x8808     /* IEEE Pause frames. See 802.3 31B */
#define ETH_P_SLOW 0x8809      /* Slow Protocol. See 802.3ad 43B */
#define ETH_P_WCCP 0x883E      /* Web-cache coordination protocol \
                                * defined in draft-wilson-wrec-wccp-v2-00.txt */
#define ETH_P_MPLS_UC 0x8847   /* MPLS Unicast traffic		*/
#define ETH_P_MPLS_MC 0x8848   /* MPLS Multicast traffic	*/
#define ETH_P_ATMMPOA 0x884c   /* MultiProtocol Over ATM	*/
#define ETH_P_PPP_DISC 0x8863  /* PPPoE discovery messages     */
#define ETH_P_PPP_SES 0x8864   /* PPPoE session messages	*/
#define ETH_P_LINK_CTL 0x886c  /* HPNA, wlan link local tunnel */
#define ETH_P_ATMFATE 0x8884   /* Frame-based ATM Transport \
                                * over Ethernet             \
                                */
#define ETH_P_PAE 0x888E       /* Port Access Entity (IEEE 802.1X) */
#define ETH_P_AOE 0x88A2       /* ATA over Ethernet		*/
#define ETH_P_8021AD 0x88A8    /* 802.1ad Service VLAN		*/
#define ETH_P_802_EX1 0x88B5   /* 802.1 Local Experimental 1.  */
#define ETH_P_PREAUTH 0x88C7   /* 802.11 Preauthentication */
#define ETH_P_TIPC 0x88CA      /* TIPC 			*/
#define ETH_P_LLDP 0x88CC      /* Link Layer Discovery Protocol */
#define ETH_P_MRP 0x88E3       /* Media Redundancy Protocol	*/
#define ETH_P_MACSEC 0x88E5    /* 802.1ae MACsec */
#define ETH_P_8021AH 0x88E7    /* 802.1ah Backbone Service Tag */
#define ETH_P_MVRP 0x88F5      /* 802.1Q MVRP                  */
#define ETH_P_1588 0x88F7      /* IEEE 1588 Timesync */
#define ETH_P_NCSI 0x88F8      /* NCSI protocol		*/
#define ETH_P_PRP 0x88FB       /* IEC 62439-3 PRP/HSRv0	*/
#define ETH_P_CFM 0x8902       /* Connectivity Fault Management */
#define ETH_P_FCOE 0x8906      /* Fibre Channel over Ethernet  */
#define ETH_P_IBOE 0x8915      /* Infiniband over Ethernet	*/
#define ETH_P_TDLS 0x890D      /* TDLS */
#define ETH_P_FIP 0x8914       /* FCoE Initialization Protocol */
#define ETH_P_80221 0x8917     /* IEEE 802.21 Media Independent Handover Protocol */
#define ETH_P_HSR 0x892F       /* IEC 62439-3 HSRv1	*/
#define ETH_P_NSH 0x894F       /* Network Service Header */
#define ETH_P_LOOPBACK 0x9000  /* Ethernet loopback packet, per IEEE 802.3 */
#define ETH_P_QINQ1 0x9100     /* deprecated QinQ VLAN [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_QINQ2 0x9200     /* deprecated QinQ VLAN [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_QINQ3 0x9300     /* deprecated QinQ VLAN [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_EDSA 0xDADA      /* Ethertype DSA [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_DSA_8021Q 0xDADB /* Fake VLAN Header for DSA [ NOT AN OFFICIALLY REGISTERED ID ] */
#define ETH_P_IFE 0xED3E       /* ForCES inter-FE LFB type */
#define ETH_P_AF_IUCV 0xFBFB   /* IBM af_iucv [ NOT AN OFFICIALLY REGISTERED ID ] */

#define ETH_P_802_3_MIN 0x0600 /* If the value in the ethernet type is less than this value   \
                                                                                            \ \
/* Header cursor to keep track of current parsing position */
struct hdr_cursor
{
    void *pos;
    void *end;
};

/*
 * Struct icmphdr_common represents the common part of the icmphdr and icmp6hdr
 * structures.
 */
struct icmphdr_common
{
    __u8 type;
    __u8 code;
    __sum16 cksum;
};

/* Allow users of header file to redefine VLAN max depth */
#ifndef VLAN_MAX_DEPTH
#define VLAN_MAX_DEPTH 2
#endif

/* Longest chain of IPv6 extension headers to resolve */
#ifndef IPV6_EXT_MAX_CHAIN
#define IPV6_EXT_MAX_CHAIN 6
#endif

#define VLAN_VID_MASK 0x0fff /* VLAN Identifier */
/* Struct for collecting VLANs after parsing via parse_ethhdr_vlan */
struct collect_vlans
{
    __u16 id[VLAN_MAX_DEPTH];
};

static __always_inline int proto_is_vlan(__u16 h_proto)
{
    return !!(h_proto == bpf_htons(ETH_P_8021Q) ||
              h_proto == bpf_htons(ETH_P_8021AD));
}

/* Notice, parse_ethhdr() will skip VLAN tags, by advancing nh->pos and returns
 * next header EtherType, BUT the ethhdr pointer supplied still points to the
 * Ethernet header. Thus, caller can look at eth->h_proto to see if this was a
 * VLAN tagged packet.
 */
static __always_inline int parse_ethhdr_vlan(struct hdr_cursor *nh,
                                             void *data_end,
                                             struct ethhdr **ethhdr,
                                             struct collect_vlans *vlans)
{
    struct ethhdr *eth = nh->pos;
    int hdrsize = sizeof(*eth);
    struct vlan_hdr *vlh;
    __u16 h_proto;
    int i;

    /* Byte-count bounds check; check if current pointer + size of header
     * is after data_end.
     */
    if (nh->pos + hdrsize > data_end)
        return -1;

    nh->pos += hdrsize;
    *ethhdr = eth;
    vlh = nh->pos;
    h_proto = eth->h_proto;

    /* Use loop unrolling to avoid the verifier restriction on loops;
     * support up to VLAN_MAX_DEPTH layers of VLAN encapsulation.
     */
#pragma unroll
    for (i = 0; i < VLAN_MAX_DEPTH; i++)
    {
        if (!proto_is_vlan(h_proto))
            break;

        if (vlh + 1 > data_end)
            break;

        h_proto = vlh->h_vlan_encapsulated_proto;
        if (vlans) /* collect VLAN ids */
            vlans->id[i] =
                (bpf_ntohs(vlh->h_vlan_TCI) & VLAN_VID_MASK);

        vlh++;
    }

    nh->pos = vlh;
    return h_proto; /* network-byte-order */
}

static __always_inline int parse_ethhdr(struct hdr_cursor *nh,
                                        void *data_end,
                                        struct ethhdr **ethhdr)
{
    /* Expect compiler removes the code that collects VLAN ids */
    return parse_ethhdr_vlan(nh, data_end, ethhdr, NULL);
}

static __always_inline int skip_ip6hdrext(struct hdr_cursor *nh,
                                          void *data_end,
                                          __u8 next_hdr_type)
{
    for (int i = 0; i < IPV6_EXT_MAX_CHAIN; ++i)
    {
        struct ipv6_opt_hdr *hdr = nh->pos;

        if (hdr + 1 > data_end)
            return -1;

        switch (next_hdr_type)
        {
        case IPPROTO_HOPOPTS:
        case IPPROTO_DSTOPTS:
        case IPPROTO_ROUTING:
        case IPPROTO_MH:
            nh->pos = (char *)hdr + (hdr->hdrlen + 1) * 8;
            next_hdr_type = hdr->nexthdr;
            break;
        case IPPROTO_AH:
            nh->pos = (char *)hdr + (hdr->hdrlen + 2) * 4;
            next_hdr_type = hdr->nexthdr;
            break;
        case IPPROTO_FRAGMENT:
            nh->pos = (char *)hdr + 8;
            next_hdr_type = hdr->nexthdr;
            break;
        default:
            /* Found a header that is not an IPv6 extension header */
            return next_hdr_type;
        }
    }

    return -1;
}

static __always_inline int parse_ip6hdr(struct hdr_cursor *nh,
                                        void *data_end,
                                        struct ipv6hdr **ip6hdr)
{
    struct ipv6hdr *ip6h = nh->pos;

    /* Pointer-arithmetic bounds check; pointer +1 points to after end of
     * thing being pointed to. We will be using this style in the remainder
     * of the tutorial.
     */
    if (ip6h + 1 > data_end)
        return -1;

    if (ip6h->version != 6)
        return -1;

    nh->pos = ip6h + 1;
    *ip6hdr = ip6h;

    return skip_ip6hdrext(nh, data_end, ip6h->nexthdr);
}

static __always_inline int parse_iphdr(struct hdr_cursor *nh,
                                       void *data_end,
                                       struct iphdr **iphdr)
{
    struct iphdr *iph = nh->pos;
    int hdrsize;

    if (iph + 1 > data_end)
        return -1;

    if (iph->version != 4)
        return -1;

    hdrsize = iph->ihl * 4;
    /* Sanity check packet field is valid */
    if (hdrsize < sizeof(*iph))
        return -1;

    /* Variable-length IPv4 header, need to use byte-based arithmetic */
    if (nh->pos + hdrsize > data_end)
        return -1;

    nh->pos += hdrsize;
    *iphdr = iph;

    return iph->protocol;
}

static __always_inline int parse_icmp6hdr(struct hdr_cursor *nh,
                                          void *data_end,
                                          struct icmp6hdr **icmp6hdr)
{
    struct icmp6hdr *icmp6h = nh->pos;

    if (icmp6h + 1 > data_end)
        return -1;

    nh->pos = icmp6h + 1;
    *icmp6hdr = icmp6h;

    return icmp6h->icmp6_type;
}

static __always_inline int parse_icmphdr(struct hdr_cursor *nh,
                                         void *data_end,
                                         struct icmphdr **icmphdr)
{
    struct icmphdr *icmph = nh->pos;

    if (icmph + 1 > data_end)
        return -1;

    nh->pos = icmph + 1;
    *icmphdr = icmph;

    return icmph->type;
}

static __always_inline int parse_icmphdr_common(struct hdr_cursor *nh,
                                                void *data_end,
                                                struct icmphdr_common **icmphdr)
{
    struct icmphdr_common *h = nh->pos;

    if (h + 1 > data_end)
        return -1;

    nh->pos = h + 1;
    *icmphdr = h;

    return h->type;
}

/*
 * parse_udphdr: parse the udp header and return the length of the udp payload
 */
static __always_inline int parse_udphdr(struct hdr_cursor *nh,
                                        void *data_end,
                                        struct udphdr **udphdr)
{
    int len;
    struct udphdr *h = nh->pos;

    if (h + 1 > data_end)
        return -1;

    nh->pos = h + 1;
    *udphdr = h;

    len = bpf_ntohs(h->len) - sizeof(struct udphdr);
    if (len < 0)
        return -1;

    return len;
}

/*
 * parse_tcphdr: parse and return the length of the tcp header
 */
static __always_inline int parse_tcphdr(struct hdr_cursor *nh,
                                        void *data_end,
                                        struct tcphdr **tcphdr)
{
    int len;
    struct tcphdr *h = nh->pos;

    if (h + 1 > data_end)
        return -1;

    len = h->doff * 4;
    /* Sanity check packet field is valid */
    if (len < sizeof(*h))
        return -1;

    /* Variable-length TCP header, need to use byte-based arithmetic */
    if (nh->pos + len > data_end)
        return -1;

    nh->pos += len;
    *tcphdr = h;

    return len;
}

#endif /* __PARSING_HELPERS_H */