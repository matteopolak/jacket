// ISA-only independent C oracle. No SDK/private implementation is included.
#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>

extern uint64_t xgetbv_read(uint64_t, uint64_t *, uint64_t *);
extern uint64_t xgetbv_read32(uint32_t);
extern uint64_t xgetbv_alias(uint64_t);
extern void xgetbv_discard(uint32_t);
extern bool xgetbv_flags(uint32_t, uint64_t, uint64_t);

static void query(uint32_t leaf, uint32_t subleaf, uint32_t out[4]) {
    __asm__ volatile("cpuid" : "=a"(out[0]), "=b"(out[1]), "=c"(out[2]), "=d"(out[3])
                     : "0"(leaf), "2"(subleaf));
}
static uint64_t oracle(uint32_t index) {
    uint32_t lo, hi;
    __asm__ volatile("xgetbv" : "=a"(lo), "=d"(hi) : "c"(index));
    return ((uint64_t)hi << 32) | lo;
}
static int check(uint32_t index, uint64_t enabled, uint64_t defined) {
    uint64_t hi = UINT64_MAX, lo = UINT64_MAX;
    uint64_t before = oracle(index);
    uint64_t actual = xgetbv_read(index, &hi, &lo);
    uint64_t after = oracle(index);
    if (hi > UINT32_MAX || lo > UINT32_MAX || actual != ((hi << 32) | lo)) return 1;
    if (index == 0 && (((actual ^ enabled) & defined) || ((actual ^ before) & defined) || ((actual ^ after) & defined))) return 2;
    // XINUSE can change inside the compiled call even if adjacent observations
    // agree. Its enabled-state subset is stable; exact equality is not promised.
    if (index == 1 && (actual & ~enabled & defined)) return 3;
    uint64_t narrow = xgetbv_read32(index);
    uint64_t alias = xgetbv_alias(UINT64_C(0x100000000) | index);
    if (index == 0 && (((narrow ^ enabled) & defined) || ((alias ^ enabled) & defined))) return 4;
    if (index == 1 && ((narrow | alias) & ~enabled & defined)) return 5;
    if (index == 0) {
        actual = xgetbv_read(UINT64_C(0x100000000), &hi, &lo);
        if (((actual ^ enabled) & defined) || hi > UINT32_MAX || lo > UINT32_MAX) return 6;
    }
    const uint64_t pairs[][2] = {{0,0},{0,1},{UINT64_MAX,1},
        {UINT64_C(0x8000000000000000),1},{UINT64_C(0x7fffffffffffffff),UINT64_MAX}};
    for (unsigned i=0;i<sizeof pairs/sizeof pairs[0];++i)
        if (!xgetbv_flags(index,pairs[i][0],pairs[i][1])) return 7;
    xgetbv_discard(index);
    return 0;
}
int main(void) {
    uint32_t info[4];
    query(0,0,info);
    uint32_t maximum = info[0];
    if (maximum < 1) { puts("UNAVAILABLE XGETBV: no CPUID leaf1"); return 77; }
    query(1,0,info);
    if ((info[2] & (UINT32_C(3)<<26)) != (UINT32_C(3)<<26)) {
        puts("UNAVAILABLE XGETBV: XSAVE/OSXSAVE absent"); return 77;
    }
    uint64_t enabled = oracle(0);
    // XCR0.x87 is mandatory, proving that fabricated zero is not valid.
    if (!(enabled & 1)) return 10;
    // SDM leaves unimplemented result bits undefined. Compare only enumerated
    // XCR0 state bits, while checking the complete output-pair ABI separately.
    uint64_t defined = 1;
    if (maximum >= 0xd) { query(0xd,0,info); defined = ((uint64_t)info[3]<<32)|info[0]; }
    int result = check(0,enabled,defined);
    if (result) return 20+result;
    bool index1 = false;
    if (maximum >= 0xd) { query(0xd,1,info); index1 = (info[0] & 4)!=0; }
    if (index1) {
        result = check(1,enabled,defined);
        if (result) return 40+result;
    }
    printf("XGETBV real index0 PASS; index1 %s\n",index1 ? "PASS" : "UNAVAILABLE");
    return 0;
}
