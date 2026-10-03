/* Dynamic C varargs forwarding through the official libffi ABI. */
#include <stdint.h>
#include <stdlib.h>
#ifdef __APPLE__
#include <ffi/ffi.h>
#else
#include <ffi.h>
#endif

typedef struct binding_argument {
    uint32_t kind;
    uint32_t reserved;
    uint64_t value;
} binding_argument;
_Static_assert(sizeof(binding_argument) == 16, "binding argument ABI");

int binding_ffi_call(void *function, int result_kind, uint32_t fixed_count,
                     binding_argument *arguments, uint32_t count, uint64_t *result) {
    if (!function || !result || fixed_count > count)
        return FFI_BAD_TYPEDEF;
    ffi_type **types = malloc(sizeof(*types) * count);
    void **values = malloc(sizeof(*values) * count);
    if ((!types || !values) && count) {
        free(types);
        free(values);
        return FFI_BAD_TYPEDEF;
    }
    for (uint32_t i = 0; i < count; ++i) {
        switch (arguments[i].kind & 255) {
            case 1: types[i] = &ffi_type_sint32; break;
            case 2: types[i] = &ffi_type_uint32; break;
            case 3: types[i] = &ffi_type_sint64; break;
            case 4: types[i] = &ffi_type_uint64; break;
            case 5: types[i] = &ffi_type_double; break;
            case 6: types[i] = &ffi_type_pointer; break;
            default: free(types); free(values); return FFI_BAD_TYPEDEF;
        }
        values[i] = &arguments[i].value;
    }
    ffi_cif cif;
    ffi_type *returns;
    switch (result_kind) {
        case 0: returns = &ffi_type_sint32; break;
        case 1: returns = &ffi_type_pointer; break;
        case 2: returns = &ffi_type_void; break;
        case 3: returns = &ffi_type_uint8; break;
        default: free(types); free(values); return FFI_BAD_TYPEDEF;
    }
    ffi_status status = ffi_prep_cif_var(&cif, FFI_DEFAULT_ABI, fixed_count, count,
                                        returns, types);
    if (status == FFI_OK) {
        ffi_arg output = 0;
        ffi_call(&cif, FFI_FN(function), result_kind == 2 ? NULL : &output, values);
        *result = (uint64_t)output;
    }
    free(types);
    free(values);
    return status;
}
