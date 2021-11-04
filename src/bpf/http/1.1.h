#include "../vmlinux.h"

struct http1_1_req_hdr_t {
    short version;
};

int parse_http1_1_req_hdr(struct http1_1_req_hdr_t *hdr, const char *buf, int len);{
    int i = 0;
    while (i < len) {
        if (buf[i] == '\r' && buf[i + 1] == '\n') {
            hdr->version = 1;
            return i + 2;
        }
        i++;
    }
    return -1;
};