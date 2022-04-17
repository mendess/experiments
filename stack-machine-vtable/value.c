#include "value.h"

#include "result.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>

long const* value_as_long(Value const* value) {
    return value->table->type == TYPE_INTEGER ? (long const*) value->bytes.buf
                                              : NULL;
}

char* const* value_as_string(Value const* value) {
    return value->table->type == TYPE_STRING ? (char* const*) value->bytes.buf
                                             : NULL;
}

// long
RuntimeResult integer_add(Bytes lhs, Value const* rhs, Stack* stack) {
    (void) stack;
    long lhs_l = b_to_long(lhs);
    long const* rhs_l = value_as_long(rhs);
    return rhs_l != NULL ? result_success(new_integer(lhs_l + *rhs_l))
                         : result_error(ERROR_UNSUPORTED_OP);
}

RuntimeResult integer_sub(Bytes lhs, Value const* rhs, Stack* stack) {
    (void) stack;
    long lhs_l = b_to_long(lhs);
    long const* rhs_l = value_as_long(rhs);
    return rhs_l != NULL ? result_success(new_integer(lhs_l - *rhs_l))
                         : result_error(ERROR_UNSUPORTED_OP);
}

RuntimeResult integer_to_int(Bytes lhs, Stack* stack) {
    (void) stack;
    long lhs_l = b_to_long(lhs);
    return result_success(new_integer(lhs_l));
}

// string
RuntimeResult string_add(Bytes lhs, Value const* rhs, Stack* stack) {
    (void) stack;
    char* lhs_s = b_to_char_star(lhs);
    char* const* rhs_s = value_as_string(rhs);
    if (rhs_s != NULL) {
        char* new_str = malloc(strlen(lhs_s) + strlen(*rhs_s) + 1);
        strcat(strcpy(new_str, lhs_s), *rhs_s);
        return result_success(new_string(new_str));
    } else {
        return result_error(ERROR_UNSUPORTED_OP);
    }
}

RuntimeResult string_sub(Bytes lhs, Value const* rhs, Stack* stack) {
    (void) stack;
    (void) lhs;
    (void) rhs;
    return result_error(ERROR_UNSUPORTED_OP);
}

RuntimeResult string_to_int(Bytes lhs, Stack* stack) {
    (void) stack;
    char* lhs_s = b_to_char_star(lhs);
    char* endptr;
    long new_int = strtol(lhs_s, &endptr, 10);
    return *endptr == '\0' ? result_success(new_integer(new_int))
                           : result_error(ERROR_PARSE_TO_INT_FAILED);
}

static ValueTable const INTEGER_TABLE = {
    .add = integer_add,
    .sub = integer_sub,
    .to_int = integer_to_int,
    .type = TYPE_INTEGER,
};

static ValueTable const STRING_TABLE = {
    .add = string_add,
    .sub = string_sub,
    .to_int = string_to_int,
    .type = TYPE_STRING,
};

Value new_integer(long i) {
    Value v = {.table = &INTEGER_TABLE};
    memcpy(v.bytes.buf, &i, sizeof i);
    return v;
}

Value new_string(char* s) {
    Value v = {.table = &STRING_TABLE};
    memcpy(v.bytes.buf, &s, sizeof s);
    return v;
}

Value new_string_from(char const* s) {
    char* buf = malloc(strlen(s) + 1);
    strcpy(buf, s);
    return new_string(buf);
}

void value_print(FILE* fp, Value const* value) {
    {
        long const* v = value_as_long(value);
        if (v != NULL) {
            fprintf(fp, "%ld", *v);
            return;
        }
    }
    {
        char* const* v = value_as_string(value);
        if (v != NULL) {
            fprintf(fp, "%s", *v);
            return;
        }
    }
}

void value_drop(Value* value) {
    char* const* maybe_str = value_as_string(value);
    if (maybe_str != NULL) {
        free(*maybe_str);
    }
}
