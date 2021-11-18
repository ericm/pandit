#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>

#define KEY_SIZE 32
#define MTU 1600
#define PROG(F)               \
    SEC("xdp/"__stringify(F)) \
    int bpf_func_##F

typedef struct
{
    __uint(type, BPF_MAP_TYPE_STACK);
    __uint(max_entries, 8192);
    __type(key, pdt_buff_t);
    __type(value, pdt_buff_t);
} pdt_hash_t;

// PROG(PDT_BUFF_FIND)
// (pdt_buff_t *a)
// {
//     return 0;
// }

// static __always_inline int
// pdt_hash_find(pdt_hash_t *hash, char *key, pdt_hash_el_t **elem)
// {
//     *elem = (pdt_hash_el_t *)bpf_map_lookup_elem(hash, key);
//     if (!(*elem))
//         return 0;
//     return 1;
// }

// static __always_inline int
// pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t *buf, pdt_buff_t *kv_sep, pdt_buff_t *el_sep)
// {
//     return 1;
// }