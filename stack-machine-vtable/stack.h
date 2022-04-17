#ifndef STACK_H
#define STACK_H

#include "result.h"
#include "value.h"

#include <stdio.h>

typedef struct Stack {
    size_t sp;
    size_t cap;
    Value* values;
} Stack;

RuntimeResult stack_pop(Stack* stack);

void stack_push(Stack* stack, Value v);

void stack_print(FILE* fp, Stack* stack);

void stack_drop(Stack* stack);

#endif
