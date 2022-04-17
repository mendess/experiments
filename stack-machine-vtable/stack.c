#include "stack.h"

#include <stdio.h>
#include <stdlib.h>

RuntimeResult stack_pop(Stack* stack) {
    return stack->sp == 0 ? result_error(ERROR_STACK_EMPTY)
                          : result_success(stack->values[--stack->sp]);
}

void stack_push(Stack* stack, Value v) {
    if (stack->sp == stack->cap) {
        stack->cap = stack->cap == 0 ? 1 : stack->cap * 2;
        stack->values = realloc(stack->values, sizeof(Value) * stack->cap);
    }
    stack->values[stack->sp++] = v;
}

void stack_print(FILE* fp, Stack* stack) {
    fprintf(fp, "[");
    if (stack->sp > 0) {
        for (size_t i = 0; i < stack->sp - 1; ++i) {
            value_print(fp, stack->values + i);
            fprintf(fp, ", ");
        }
        value_print(fp, stack->values + (stack->sp - 1));
    }
    fprintf(fp, "]\n");
}

void stack_drop(Stack* stack) {
    Value const* const end = stack->values + stack->sp;
    for (Value* i = stack->values; i != end; ++i) {
        value_drop(i);
    }
    free(stack->values);
}
