#ifndef ORIGINAL_BITFIELD_PRINTERS_H
#define ORIGINAL_BITFIELD_PRINTERS_H
struct OriginalBits {
    unsigned first : 3;
    unsigned second : 4;
    unsigned on : 1;
    unsigned : 0;
    unsigned final : 3;
};
void original_bits_initialize(struct OriginalBits *, unsigned, unsigned, unsigned, unsigned);
unsigned original_bits_read(const struct OriginalBits *);
struct OriginalScalarBits { unsigned a : 3; unsigned b : 5; };
void original_scalar_initialize(struct OriginalScalarBits *, unsigned, unsigned);
unsigned original_scalar_read(const struct OriginalScalarBits *);
#endif
