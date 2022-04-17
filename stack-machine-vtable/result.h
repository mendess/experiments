#ifndef RESULT_H
#define RESULT_H

#include "value.h"

typedef enum Error {
    ERROR_UNSUPORTED_OP,
    ERROR_STACK_EMPTY,
    ERROR_PARSE_TO_INT_FAILED
} Error;

typedef enum ResultKind {
    RESULT_KIND_ERROR,
    RESULT_KIND_VALUE,
    RESULT_KIND_VOID,
} ResultKind;

typedef struct RuntimeResult {
    ResultKind kind;
    union {
        Value v;
        Error e;
    } content;
} RuntimeResult;

RuntimeResult result_success(Value v);

RuntimeResult result_error(Error e);

extern RuntimeResult result_void;

#endif
