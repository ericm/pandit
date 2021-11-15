#include "hash.h"

int pdt_buff_find(pdt_buff_t a, pdt_buff_t b)
{
    __u16 i, j;
    for (i = 0; i < a.size; i++)
    {
        for (j = 0; j < b.size; j++)
        {
            if (a.buf[i + j] != b.buf[j])
            {
                break;
            }
            if (j == b.size - 1)
            {
                return i;
            }
        }
    }
    return -1;
}

int pdt_hash_find(pdt_hash_t *hash, char *key, pdt_hash_el_t **elem)
{
    *elem = (pdt_hash_el_t *)bpf_map_lookup_elem(hash, key);
    if (!(*elem))
    {
        return 0;
    }
    return 1;
}

int pdt_hash_populate(pdt_hash_t *hash, pdt_buff_t buf, pdt_buff_t kv_sep, pdt_buff_t el_sep)
{
}