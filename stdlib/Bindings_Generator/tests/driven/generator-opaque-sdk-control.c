#include "generator-opaque-printer.h"
#include <assert.h>

_Static_assert(sizeof(struct Forward) == 4, "defined native record size");
_Static_assert(_Alignof(struct Forward) == 4, "defined native record alignment");

int main(void) {
    struct Opaque *pointer = opaque_pointer();
    assert(pointer && opaque_identity(pointer) == pointer);
    assert(opaque_value(pointer) == 73);
    union OpaqueUnion *union_pointer = opaque_union_pointer();
    assert(union_pointer && opaque_union_value(union_pointer) == 91);
    assert(opaque_native_by_value_control() == 73);
    struct Forward forward = {29};
    assert(forward_value(&forward) == 29);
    return 0;
}
