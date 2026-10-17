#include "generator-model-boundaries.hpp"
#include <cstddef>
#include <cstdio>
static_assert(sizeof(BoundaryContainer) == 8 && alignof(BoundaryContainer) == 4);
static_assert(offsetof(BoundaryContainer, inner) == 0 && offsetof(BoundaryContainer, tail) == 4);
static_assert(sizeof(BoundaryBits) == 4 && alignof(BoundaryBits) == 4);
static_assert(sizeof(BoundaryPolyBits) == 16 && alignof(BoundaryPolyBits) == 8);
static_assert(sizeof(BoundaryUnsupportedField) == 8);
int boundary_call() { return 7; }
int main() {
    BoundaryContainer value{{11}, 13};
    BoundaryBits bits{5, 21};
    if (value.inner.payload != 11 || value.tail != 13 || bits.first != 5 || bits.second != 21 || rejected_value != 7) return 1;
    std::puts("ok");
}
