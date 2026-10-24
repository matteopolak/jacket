#include <stddef.h>
#include <string.h>

#include "bindings-bitfield-layout.h"

// The C compiler's own view of each record in bindings-bitfield-layout.h: sizes, alignments, the
// offsets of ordinary members, and every bitfield read and written through C.

typedef struct {
    const char *key;
    long long value;
} Fact;

#define RECORDS(X) \
    X(struct BfIssue, BfIssue) \
    X(struct BfSigned, BfSigned) \
    X(struct BfBool, BfBool) \
    X(struct BfEnum, BfEnum) \
    X(struct BfUnnamed, BfUnnamed) \
    X(struct BfZeroWidth, BfZeroWidth) \
    X(struct BfStraddle, BfStraddle) \
    X(struct BfWide, BfWide) \
    X(struct BfTrail, BfTrail) \
    X(struct BfGap, BfGap) \
    X(struct BfComments, BfComments) \
    X(struct BfInner, BfInner) \
    X(struct BfNested, BfNested) \
    X(struct BfClash, BfClash) \
    X(struct BfAnonymous, BfAnonymous) \
    X(union BfUnion, BfUnion) \
    X(union BfUnionBits, BfUnionBits) \
    X(struct BfPack1, BfPack1) \
    X(struct BfPack2, BfPack2) \
    X(struct BfPackedBits, BfPackedBits) \
    X(union BfPackedUnion, BfPackedUnion)

#define OFFSETS(X) \
    X(struct BfIssue, BfIssue, section) \
    X(struct BfIssue, BfIssue, offset) \
    X(struct BfZeroWidth, BfZeroWidth, tail) \
    X(struct BfWide, BfWide, after) \
    X(struct BfTrail, BfTrail, before) \
    X(struct BfTrail, BfTrail, after) \
    X(struct BfGap, BfGap, between) \
    X(struct BfComments, BfComments, tail) \
    X(struct BfNested, BfNested, inner) \
    X(struct BfAnonymous, BfAnonymous, before) \
    X(union BfUnion, BfUnion, whole) \
    X(union BfUnion, BfUnion, parts) \
    X(struct BfPack1, BfPack1, a) \
    X(struct BfPack1, BfPack1, b) \
    X(struct BfPack1, BfPack1, c) \
    X(struct BfPack2, BfPack2, a) \
    X(struct BfPack2, BfPack2, b) \
    X(struct BfPackedBits, BfPackedBits, a) \
    X(struct BfPackedBits, BfPackedBits, z) \
    X(union BfPackedUnion, BfPackedUnion, byte)

// X(record, name, id, member path, member type)
#define FIELDS(X) \
    X(struct BfIssue, BfIssue, type, type, uint32_t) \
    X(struct BfIssue, BfIssue, size, size, uint32_t) \
    X(struct BfSigned, BfSigned, a, a, int8_t) \
    X(struct BfSigned, BfSigned, b, b, int16_t) \
    X(struct BfSigned, BfSigned, c, c, uint32_t) \
    X(struct BfSigned, BfSigned, d, d, int32_t) \
    X(struct BfSigned, BfSigned, e, e, int64_t) \
    X(struct BfSigned, BfSigned, f, f, uint64_t) \
    X(struct BfBool, BfBool, flag, flag, bool) \
    X(struct BfBool, BfBool, other, other, _Bool) \
    X(struct BfBool, BfBool, rest, rest, uint8_t) \
    X(struct BfBool, BfBool, last, last, bool) \
    X(struct BfEnum, BfEnum, color, color, enum BfColor) \
    X(struct BfEnum, BfEnum, level, level, enum BfLevel) \
    X(struct BfEnum, BfEnum, on, on, bool) \
    X(struct BfEnum, BfEnum, small, small, int) \
    X(struct BfUnnamed, BfUnnamed, a, a, uint32_t) \
    X(struct BfUnnamed, BfUnnamed, b, b, uint32_t) \
    X(struct BfUnnamed, BfUnnamed, c, c, uint32_t) \
    X(struct BfUnnamed, BfUnnamed, d, d, uint32_t) \
    X(struct BfZeroWidth, BfZeroWidth, a, a, uint8_t) \
    X(struct BfZeroWidth, BfZeroWidth, b, b, uint8_t) \
    X(struct BfZeroWidth, BfZeroWidth, c, c, uint16_t) \
    X(struct BfStraddle, BfStraddle, a, a, uint8_t) \
    X(struct BfStraddle, BfStraddle, b, b, uint8_t) \
    X(struct BfStraddle, BfStraddle, c, c, uint16_t) \
    X(struct BfStraddle, BfStraddle, d, d, uint16_t) \
    X(struct BfStraddle, BfStraddle, e, e, uint32_t) \
    X(struct BfStraddle, BfStraddle, f, f, uint32_t) \
    X(struct BfWide, BfWide, whole, whole, uint64_t) \
    X(struct BfWide, BfWide, signed_whole, signed_whole, int64_t) \
    X(struct BfWide, BfWide, high, high, uint64_t) \
    X(struct BfWide, BfWide, low, low, uint64_t) \
    X(struct BfTrail, BfTrail, bits, bits, uint32_t) \
    X(struct BfGap, BfGap, a, a, uint8_t) \
    X(struct BfGap, BfGap, b, b, uint8_t) \
    X(struct BfGap, BfGap, c, c, uint8_t) \
    X(struct BfComments, BfComments, count, count, uint16_t) \
    X(struct BfComments, BfComments, flag, flag, uint16_t) \
    X(struct BfComments, BfComments, level, level, uint16_t) \
    X(struct BfInner, BfInner, x, x, int8_t) \
    X(struct BfInner, BfInner, y, y, uint8_t) \
    X(struct BfNested, BfNested, head, head, uint16_t) \
    X(struct BfNested, BfNested, foot, foot, uint16_t) \
    X(struct BfClash, BfClash, x, x, int) \
    X(struct BfClash, BfClash, y, y, int) \
    X(struct BfAnonymous, BfAnonymous, p, p, uint8_t) \
    X(struct BfAnonymous, BfAnonymous, q, q, uint8_t) \
    X(union BfUnion, BfUnion, parts_lo, parts.lo, uint32_t) \
    X(union BfUnion, BfUnion, parts_hi, parts.hi, uint32_t) \
    X(union BfUnion, BfUnion, narrow, narrow, uint16_t) \
    X(union BfUnionBits, BfUnionBits, a, a, uint32_t) \
    X(union BfUnionBits, BfUnionBits, b, b, int32_t) \
    X(struct BfPack1, BfPack1, f, f, uint32_t) \
    X(struct BfPack1, BfPack1, g, g, uint32_t) \
    X(struct BfPack2, BfPack2, f, f, uint16_t) \
    X(struct BfPack2, BfPack2, g, g, uint16_t) \
    X(struct BfPack2, BfPack2, h, h, uint32_t) \
    X(struct BfPackedBits, BfPackedBits, x, x, uint32_t) \
    X(struct BfPackedBits, BfPackedBits, y, y, uint32_t) \
    X(union BfPackedUnion, BfPackedUnion, wide, wide, uint32_t)

#define SIZE_FACT(record, name) {#name ".size", (long long) sizeof(record)}, {#name ".align", (long long) _Alignof(record)},
#define OFFSET_FACT(record, name, member) {#name "." #member, (long long) offsetof(record, member)},

static const Fact facts[] = {RECORDS(SIZE_FACT) OFFSETS(OFFSET_FACT)};

#define ACCESSORS(record, name, id, path, type) \
    static long long get_##name##_##id(const void *p) { return (long long) ((const record *) p)->path; } \
    static void set_##name##_##id(void *p, long long v) { ((record *) p)->path = (type) v; }

FIELDS(ACCESSORS)

typedef struct {
    const char *key;
    long long (*get)(const void *);
    void (*set)(void *, long long);
} Field;

#define FIELD_ENTRY(record, name, id, path, type) {#name "." #id, get_##name##_##id, set_##name##_##id},

static const Field fields[] = {FIELDS(FIELD_ENTRY)};

static const Field *find_field(const char *key) {
    for (size_t i = 0; i < sizeof(fields) / sizeof(fields[0]); i++) {
        if (strcmp(fields[i].key, key) == 0) return &fields[i];
    }

    return NULL;
}

// A size, alignment or member offset in bytes; -1 when the record has no such fact.
long long oracle_fact(const char *key) {
    for (size_t i = 0; i < sizeof(facts) / sizeof(facts[0]); i++) {
        if (strcmp(facts[i].key, key) == 0) return facts[i].value;
    }

    return -1;
}

long long oracle_field_count(void) { return (long long) (sizeof(fields) / sizeof(fields[0])); }

long long oracle_get(const char *key, const void *record) {
    const Field *field = find_field(key);

    return field ? field->get(record) : 0;
}

long long oracle_set(const char *key, void *record, long long value) {
    const Field *field = find_field(key);
    if (!field) return 0;

    field->set(record, value);

    return 1;
}
