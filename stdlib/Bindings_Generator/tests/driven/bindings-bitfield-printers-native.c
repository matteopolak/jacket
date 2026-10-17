#include "bindings-bitfield-printers.h"
void original_bits_initialize(struct OriginalBits *bits, unsigned first,
                              unsigned second, unsigned on, unsigned final) {
    bits->first = first;
    bits->second = second;
    bits->on = on;
    bits->final = final;
}
unsigned original_bits_read(const struct OriginalBits *bits) {
    return bits->first | (bits->second << 3) | (bits->on << 7) | (bits->final << 8);
}
void original_scalar_initialize(struct OriginalScalarBits *bits, unsigned a, unsigned b) {
    bits->a = a;
    bits->b = b;
}
unsigned original_scalar_read(const struct OriginalScalarBits *bits) {
    return bits->a | (bits->b << 3);
}
