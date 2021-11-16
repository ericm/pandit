#include "../vmlinux.h"
#include <bpf/bpf_helpers.h>

#define KEY_SIZE 32

#pragma pack(16)
typedef struct pdt_buff
{
    __u8 *buf;
    __u8 offset;
    __u8 size;
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
    __type(key, pdt_buff_t);
    __type(value, pdt_buff_t);
} pdt_hash_t;

static __always_inline __u8
pdt_buff_find(pdt_buff_t *a, pdt_buff_t *b)
{
    __u8 i = 0, j, t_j;
    __u8 *a_buf, *b_buf;
    __u8 *a_cmp, *b_cmp;
    if (!a || !b)
        return -1;
    if (a->size < b->size)
        return -1;
    a_buf = a->buf;
    b_buf = b->buf;
    if (!a_buf || !b_buf)
        return 0;
    if (b->size == 0)
        return 0;

    for (i = a->offset; i < a->size; i++)
    {
        for (j = b->offset; j < b->size; j++)
        {
            if (i + j > a->size - 1)
                return 0;
            a_cmp = a_buf + i + j;
            b_cmp = b_buf + j;
            if (!a_cmp || !b_cmp)
                return 0;
            bpf_printk("a: %d b: %d", *a_cmp, *b_cmp);
            if (__bpf_memcmp(a_cmp, b_cmp, 1))
                break;

            if (j == b->size - 1)
                return i;
        }
    }
    return 0;
}

static __always_inline int
pdt_hash_find(pdt_hash_t *hash, char *key, pdt_hash_el_t **elem)
{
    *elem = (pdt_hash_el_t *)bpf_map_lookup_elem(hash, key);
    if (!(*elem))
        return 0;
    return 1;
}

static __always_inline int
pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t *buf, pdt_buff_t *kv_sep, pdt_buff_t *el_sep)
{
    __u8 i, j, offset;
    __u8 i_kv, i_el;
    pdt_buff_t key = {.offset = 0}, value = {.offset = 0};

    if (!buf)
        return -1;
    if (!hash)
        return -1;
    offset = buf->offset;

    for (i = 0; i < buf->size; i++)
    {
        bpf_printk("parse buff");
        if (buf->offset > buf->size - 1)
            return 1;
        i_kv = pdt_buff_find(buf, kv_sep);
        if (i_kv == 0)
            return 1;
        i_el = pdt_buff_find(buf, el_sep);
        if (i_el == 0)
            return 1;

        // __u8 key[KEY_SIZE];
        // __bpf_memcpy(&key, buf->buf + buf->offset, KEY_SIZE);
        // __bpf_memzero(&key[buf->offset], 1);
        // // for (j = buf->offset; j < KEY_SIZE; j++)
        // // {
        // // }

        bpf_printk("i_kv %d i_el %d", i_kv, i_el);

        key.buf = buf->buf + buf->offset;
        key.size = i_kv;
        value.buf = buf->buf + buf->offset + i_kv + 1;
        value.size = i_el;
        // pdt_buff_t key = {.buf = buf->buf + buf->offset, .size = i_kv, .offset = 0};
        // pdt_buff_t value = {.buf = buf->buf + buf->offset + i_kv + 1, .size = i_el - i_kv, .offset = 0};
        // __bpf_memzero(&key, sizeof(pdt_buff_t));
        // __bpf_memzero(&value, sizeof(pdt_buff_t));

        bpf_map_update_elem(hash, &key, &value, BPF_ANY);
        bpf_printk("buff %d", *buf->buf);

        buf->offset = offset + i_el + el_sep->size;
    }
    return 1;
}