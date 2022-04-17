#ifndef BYTES_H
#define BYTES_H

#include <sys/types.h>

typedef struct Bytes {
    u_int8_t buf[sizeof(void*)];
} Bytes;

long b_to_long(Bytes b);

char* b_to_char_star(Bytes b);

#endif
