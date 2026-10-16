#include "generator-opaque-printer.h"

/* This definition is deliberately absent from the generator's public input. */
struct Opaque { int private_value; };
static struct Opaque instance = {73};
union OpaqueUnion { int private_value; long long other; };
static union OpaqueUnion union_instance = {91};

struct Opaque *opaque_pointer(void) { return &instance; }
struct Opaque *opaque_identity(struct Opaque *value) { return value; }
int opaque_value(const struct Opaque *value) { return value->private_value; }
int forward_value(const struct Forward *value) { return value->value; }
union OpaqueUnion *opaque_union_pointer(void) { return &union_instance; }
int opaque_union_value(const union OpaqueUnion *value) { return value->private_value; }
struct Opaque opaque_by_value_return(void) { return instance; }
int opaque_by_value_argument(struct Opaque value) { return value.private_value; }
int opaque_native_by_value_control(void) {
    return opaque_by_value_argument(opaque_by_value_return());
}
