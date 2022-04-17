#include "result.h"
#include "stack.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define UNARY_OP_SENTINEL 0x8000
#define BINARY_OP_SENTINEL 0x9000
#define EXTRACT_ARITY(token) (token & 0xF000)

typedef enum Token {
    TOKEN_TO_INT = UNARY_OP_SENTINEL,
    TOKEN_ADD = BINARY_OP_SENTINEL,
    TOKEN_SUB,
} Token;

RuntimeResult exec_bin_op(Stack* stack, Value lhs, Value rhs, Token token) {
    RuntimeResult result;
    switch (token) {
        case TOKEN_ADD:
            result = lhs.table->add(lhs.bytes, &rhs, stack);
            break;
        case TOKEN_SUB:
            result = lhs.table->sub(lhs.bytes, &rhs, stack);
            break;
        default:
            fprintf(stderr, "not a binary op %d\n", token);
            abort();
    }
    value_drop(&lhs);
    value_drop(&rhs);
    return result;
}

RuntimeResult exec_un_op(Stack* stack, Value v, Token token) {
    RuntimeResult result;
    switch (token) {
        case TOKEN_TO_INT:
            result = v.table->to_int(v.bytes, stack);
            break;
        default:
            fprintf(stderr, "not a unary op %d\n", token);
            abort();
    }
    value_drop(&v);
    return result;
}

void process_token(Stack* stack, Token token) {
    RuntimeResult result;
    switch (EXTRACT_ARITY(token)) {
        case BINARY_OP_SENTINEL: {
            RuntimeResult rv_rhs = stack_pop(stack);
            RuntimeResult rv_lhs = stack_pop(stack);
            if (rv_rhs.kind == RESULT_KIND_ERROR) {
                result = rv_rhs;
            } else if (rv_lhs.kind == RESULT_KIND_ERROR) {
                result = rv_lhs;
            } else {
                result = exec_bin_op(
                    stack, rv_lhs.content.v, rv_rhs.content.v, token);
            }
            break;
        }
        case UNARY_OP_SENTINEL: {
            result = stack_pop(stack);
            if (result.kind == RESULT_KIND_VALUE) {
                result = exec_un_op(stack, result.content.v, token);
            }
            break;
        }
    }
    switch (result.kind) {
        case RESULT_KIND_VALUE:
            stack_push(stack, result.content.v);
            break;
        case RESULT_KIND_ERROR:
            fprintf(stderr, "error. code: %d\n", result.content.e);
            break;
        case RESULT_KIND_VOID:
            break;
    }
}

int main(void) {
    Stack stack = {0};
    stack_print(stderr, &stack);

    stack_push(&stack, new_integer(1));
    stack_print(stderr, &stack);

    stack_push(&stack, new_integer(2));
    stack_print(stderr, &stack);

    stack_push(&stack, new_integer(3));
    stack_print(stderr, &stack);

    process_token(&stack, TOKEN_ADD);
    stack_print(stderr, &stack);

    process_token(&stack, TOKEN_SUB);
    stack_print(stderr, &stack);

    {
        Value v = stack_pop(&stack).content.v;
        stack_print(stderr, &stack);

        printf("1 - (2 + 3) = %ld\n", *value_as_long(&v));
        value_drop(&v);
    }
    stack_push(&stack, new_string_from("Hello "));
    stack_print(stderr, &stack);

    stack_push(&stack, new_string_from("World"));
    stack_print(stderr, &stack);

    process_token(&stack, TOKEN_ADD);
    stack_print(stderr, &stack);

    {
        Value v = stack_pop(&stack).content.v;
        stack_print(stderr, &stack);

        printf("str = %s\n", *value_as_string(&v));
        value_drop(&v);
    }

    stack_push(&stack, new_string_from("2"));
    stack_print(stderr, &stack);

    process_token(&stack, TOKEN_TO_INT);
    stack_print(stderr, &stack);
    {
        Value v = stack_pop(&stack).content.v;
        stack_print(stderr, &stack);

        printf("\"2\" = %ld\n", *value_as_long(&v));
        value_drop(&v);
    }

    stack_drop(&stack);
}
