#ifndef VALUE_H
#define VALUE_H

#include "bytes.h"

#include <stdio.h>
#include <sys/types.h>

typedef struct Stack Stack;
typedef struct RuntimeResult RuntimeResult;
typedef struct Value Value;

typedef RuntimeResult (*BinOp)(Bytes, Value const*, Stack*);
typedef RuntimeResult (*UnaryOp)(Bytes, Stack*);

typedef enum Type {
    TYPE_INTEGER,
    TYPE_STRING,
} Type;

typedef struct ValueTable {
    BinOp add;
    BinOp sub;
    UnaryOp to_int;
    Type type;
} ValueTable;

struct Value {
    ValueTable const* table;
    Bytes bytes;
};

Value new_integer(long i);

Value new_string(char* s);

Value new_string_from(char const* s);

long const* value_as_long(Value const* value);

char* const* value_as_string(Value const* value);

void value_print(FILE* fp, Value const* value);

void value_drop(Value* value);

#endif
