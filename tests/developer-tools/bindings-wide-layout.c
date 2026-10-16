#include "bindings-wide-layout.h"
#include <stddef.h>
_Static_assert(sizeof(struct IndependentWide) == 64, "original Mac64 SDK size");
_Static_assert(offsetof(struct IndependentWide, signed_value) == 16, "SDK wide alignment");
_Static_assert(offsetof(struct IndependentWide, unsigned_value) == 32, "SDK unsigned alignment");
int original_wide_exchange(struct IndependentWide *value) {
    if (value->signed_value != (((__int128)1 << 64) | 42) ||
        value->unsigned_value != ((unsigned __int128)17 << 64)) return 0;
    value->tag = 11;
    value->signed_value = ((__int128)2 << 64) | 25;
    value->unsigned_value = ((unsigned __int128)3 << 64) | 7;
    value->tail = 9;
    return 1;
}
