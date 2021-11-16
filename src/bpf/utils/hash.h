#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>

typedef struct
{
    __u8 *buf;
    __u16 offset;
    __u16 size;
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
    __type(key, pdt_buff_t *);
    __type(value, pdt_buff_t *);
} pdt_hash_t;

static __always_inline int
pdt_buff_find(pdt_buff_t *a, pdt_buff_t *b)
{
    int i, j;
    char *a_buf, *b_buf;
    char *a_cmp, *b_cmp;
    if (!a || !b)
        return -1;
    if (a->size < b->size)
        return -1;
    a_buf = (char *)a->buf;
    b_buf = (char *)b->buf;
    if (!a_buf || !b_buf)
        return -1;

    for (i = a->offset; i < a->size; i++)
    {
        for (j = b->offset; j < b->size; j++)
        {
            a_cmp = a_buf + i + j;
            b_cmp = b_buf + j;
            if (!a_cmp || !b_cmp)
                return -1;
            if (*a_cmp != *b_cmp)
                break;
            if (j == b->size - 1)
                return i;
        }
    }
    return -1;
}

int pdt_hash_find(pdt_hash_t *hash, char *key, pdt_hash_el_t **elem)
{
    *elem = (pdt_hash_el_t *)bpf_map_lookup_elem(hash, key);
    if (!(*elem))
        return 0;
    return 1;
}

int pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t *buf, pdt_buff_t *kv_sep, pdt_buff_t *el_sep)
{
    __u8 i;
    int i_kv, i_el;

    if (!buf)
        return -1;
    if (!hash)
        return -1;

    for (i = 0; i < buf->size; i++)
    {
        if (buf->offset >= buf->size)
            return 1;
        i_kv = pdt_buff_find(buf, kv_sep);
        if (i_kv == -1)
            return 1;
        i_el = pdt_buff_find(buf, el_sep);
        if (i_el == -1)
            return 1;

        pdt_buff_t key = {.buf = buf->buf + buf->offset, .size = i_kv, .offset = 0};
        pdt_buff_t value = {.buf = buf->buf + buf->offset + i_kv + 1, .size = i_el - i_kv, .offset = 0};
        // create an ebpf map

        bpf_map_update_elem(hash, &key, &value, BPF_ANY);

        buf->offset += i_el + el_sep->size;
    }
    return 1;
}