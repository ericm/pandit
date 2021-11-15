#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>

typedef struct
{
    char *buf;
    size_t offset;
    size_t size;
} pdt_buff_t;

typedef struct
{
    void *value;
    __u16 len;
} pdt_hash_el_t;

typedef struct
{
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 8192);
    __type(key, char *);
    __type(value, pdt_hash_t);
} pdt_hash_t;

int pdt_buff_find(pdt_buff_t *a, pdt_buff_t *b);

int pdt_hash_find(pdt_hash_t *hash, char *key, pdt_hash_el_t *elem);

int pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t *buf, pdt_buff_t *kv_sep, pdt_buff_t *el_sep);