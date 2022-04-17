#include "result.h"

RuntimeResult result_success(Value v) {
    return (RuntimeResult){
        .kind = RESULT_KIND_VALUE,
        .content.v = v,
    };
}

RuntimeResult result_error(Error e) {
    return (RuntimeResult){
        .kind = RESULT_KIND_ERROR,
        .content.e = e,
    };
}

RuntimeResult result_void = {
    .kind = RESULT_KIND_VOID,
};
