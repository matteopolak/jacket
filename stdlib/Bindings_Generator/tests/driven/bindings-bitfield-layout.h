#ifndef BINDINGS_BITFIELD_LAYOUT_H
#define BINDINGS_BITFIELD_LAYOUT_H
// No system headers: generating for other targets must not depend on the host's C library.
typedef __INT8_TYPE__ int8_t;
typedef __INT16_TYPE__ int16_t;
typedef __INT32_TYPE__ int32_t;
typedef __INT64_TYPE__ int64_t;
typedef __UINT8_TYPE__ uint8_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
#define bool _Bool

#define BfClash_x 5

enum BfColor { BF_RED, BF_GREEN, BF_BLUE };
enum BfLevel { BF_LOW = -2, BF_MID = 0, BF_HIGH = 1 };

struct BfIssue {
    uint8_t **section;
    uint32_t type : 8; // type of the operand data.
    uint32_t size : 24;
    uint32_t offset;
};

struct BfSigned {
    int8_t a : 3;
    int16_t b : 5;
    uint32_t c : 7;
    int32_t d : 17;
    int64_t e : 33;
    uint64_t f : 31;
};

struct BfBool {
    bool flag : 1;
    _Bool other : 1;
    uint8_t rest : 6;
    bool last : 1;
};

struct BfEnum {
    enum BfColor color : 2;
    enum BfLevel level : 3;
    bool on : 1;
    int small : 2;
};

struct BfUnnamed {
    uint32_t a : 3;
    uint32_t : 5;
    uint32_t b : 4;
    uint32_t : 0;
    uint32_t c : 2;
    int : 3;
    uint32_t d : 9;
};

struct BfZeroWidth {
    uint8_t a : 3;
    uint8_t : 0;
    uint8_t b : 2;
    uint16_t : 0;
    uint16_t c : 11;
    uint32_t tail;
};

struct BfStraddle {
    uint8_t a : 6;
    uint8_t b : 6;
    uint16_t c : 6;
    uint16_t d : 12;
    uint32_t e : 20;
    uint32_t f : 20;
};

struct BfWide {
    uint64_t whole : 64;
    int64_t signed_whole : 64;
    uint64_t high : 40;
    uint64_t low : 24;
    uint32_t after;
};

struct BfTrail {
    char before;
    uint32_t bits : 3;
    char after;
};

struct BfGap {
    uint8_t a : 1;
    int32_t between;
    uint8_t b : 3;
    uint8_t c : 7;
};

struct BfComments {
    /* number of items */
    uint16_t count : 10; // postfix on count
    // prefix line one
    // prefix line two
    uint16_t flag : 1;
    /* an opener /* inside a comment */
    uint16_t level : 5;
    uint32_t tail;
};

struct BfInner {
    int8_t x : 4;
    uint8_t y : 4;
};

struct BfNested {
    uint16_t head : 5;
    struct BfInner inner;
    uint16_t foot : 9;
};

struct BfClash {
    int x : 3;
    int y : 4;
};

struct BfAnonymous {
    int before;
    union {
        struct {
            uint8_t p : 1;
            uint8_t q : 2;
        };
        uint8_t both;
    };
};

union BfUnion {
    uint32_t whole;
    struct {
        uint32_t lo : 8;
        uint32_t hi : 24;
    } parts;
    uint16_t narrow : 9;
};

union BfUnionBits {
    uint32_t a : 5;
    int32_t b : 17;
    uint8_t c;
};

#pragma pack(push, 1)
struct BfPack1 {
    char a;
    int32_t b;
    uint32_t f : 3;
    uint32_t g : 13;
    int16_t c;
};
#pragma pack(pop)

#pragma pack(push, 2)
struct BfPack2 {
    char a;
    int32_t b;
    uint16_t f : 5;
    uint16_t g : 9;
    uint32_t h : 20;
};
#pragma pack(pop)

struct __attribute__((packed)) BfPackedBits {
    uint8_t a;
    uint32_t x : 4;
    uint32_t y : 12;
    uint8_t z;
};

union __attribute__((packed)) BfPackedUnion {
    uint32_t wide : 21;
    uint8_t byte;
};

#endif
