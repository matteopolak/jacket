#include "bindings-sdk.h"
int64_t sized_sum(struct Sized *value) { return value->x + (int64_t)value->y; }
