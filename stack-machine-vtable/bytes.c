#include "bytes.h"

#define B_TO(type, bytes) (*(type*) &bytes)

long b_to_long(Bytes b) {
    return B_TO(long, b);
}

char* b_to_char_star(Bytes b) {
    return B_TO(char*, b);
}
