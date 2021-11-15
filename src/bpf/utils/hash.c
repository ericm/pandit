#include "hash.h"

static __always_inline __u64
pdt_buff_find(pdt_buff_t a, pdt_buff_t b)
{
    __u64 i, j;
    for (i = a.offset; i < a.size; i++)
    {
        for (j = b.offset; j < b.size; j++)
        {
            if (a.buf[i + j] != b.buf[j])
                break;
            if (j == b.size - 1)
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

int pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t buf, pdt_buff_t kv_sep, pdt_buff_t el_sep)
{
    __u8 i;
    __u64 i_kv, i_el;

    for (i = 0; i < buf.size; i++)
    {
        if (buf.offset >= buf.size)
            return 1;
        i_kv = pdt_buff_find(buf, kv_sep);
        if (i_kv == -1)
            return 1;
        i_el = pdt_buff_find(buf, el_sep);
        if (i_el == -1)
            return 1;

        pdt_buff_t key = {.buf = buf.offset, .size = i_kv};
        pdt_buff_t value = {.buf = buf.offset + i_kv + 1, .size = i_el - i_kv};
        bpf_map_update_elem(hash, &key, &value, BPF_ANY);

        buf.offset += i_el + el_sep.size;
    }
}