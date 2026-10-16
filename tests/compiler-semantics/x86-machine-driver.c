// Independent native ISA oracle. Optional instructions execute only after real CPUID.
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#if defined(_MSC_VER)
#include <intrin.h>
#else
#include <cpuid.h>
#endif

extern uint64_t machine_tsc(void);
extern uint64_t machine_tscp(uint32_t *aux);
extern void machine_fences(void);
extern void machine_prefetch(uint64_t *value);
extern uint32_t machine_random16(uint64_t *value);
extern uint32_t machine_random32(uint64_t *value);
extern uint32_t machine_random64(uint64_t *value);
extern uint32_t machine_seed16(uint64_t *value);
extern uint32_t machine_seed32(uint64_t *value);
extern uint32_t machine_seed64(uint64_t *value);

static void query(uint32_t leaf, uint32_t subleaf, uint32_t out[4]) {
#if defined(_MSC_VER)
    int data[4];
    __cpuidex(data, (int)leaf, (int)subleaf);
    for (int i = 0; i < 4; ++i) out[i] = (uint32_t)data[i];
#else
    __cpuid_count(leaf, subleaf, out[0], out[1], out[2], out[3]);
#endif
}

static uint64_t counter(void) {
    uint32_t lo, hi;
    __asm__ volatile("lfence; rdtsc; lfence" : "=a"(lo), "=d"(hi) :: "memory");
    return ((uint64_t)hi << 32) | lo;
}

static uint64_t counter_aux(uint32_t *aux) {
    uint32_t lo, hi, id;
    __asm__ volatile("lfence; rdtscp; lfence" : "=a"(lo), "=d"(hi), "=c"(id) :: "memory");
    *aux = id;
    return ((uint64_t)hi << 32) | lo;
}

static unsigned oracle_random(uint64_t *value, int seed) {
    unsigned char carry;
    uint64_t result;
    if (seed) {
        __asm__ volatile("rdseed %0; setc %1" : "=&r"(result), "=&q"(carry) :: "cc");
    } else {
        __asm__ volatile("rdrand %0; setc %1" : "=&r"(result), "=&q"(carry) :: "cc");
    }
    *value = result;
    return carry;
}

static int random_checks(uint32_t (*operation)(uint64_t *), unsigned width) {
    unsigned successes = 0, failures = 0;
    for (unsigned i = 0; i < 2048; ++i) {
        uint64_t value = 0;
        uint32_t flags = operation(&value);
        if (flags > 1) {
            fprintf(stderr, "width %u: arithmetic flags must clear, got %u\n", width, flags);
            return 1; // ZF/SF/OF/PF must clear, regardless of CF.
        }
        if (width == 16 && (value & UINT64_C(0xffffffffffff0000)) != UINT64_C(0x123456789abc0000)) return 2;
        if (width == 32 && value >> 32) return 3;
        uint64_t low = width == 64 ? value : value & ((UINT64_C(1) << width) - 1);
        if (!flags && low) return 4; // ISA failure writes zero, not a fabricated success.
        if (flags) ++successes; else ++failures;
    }
    printf("width %u: %u successes, %u actual failures\n", width, successes, failures);
    return 0; // A particular success rate or nonzero random value is not an ISA promise.
}

int main(void) {
    uint32_t r[4];
    query(0, 0, r);
    uint32_t max = r[0];
    unsigned tsc = 0, rand = 0, seed = 0, tscp = 0;
    if (max >= 1) { query(1, 0, r); tsc = (r[3] >> 4) & 1; rand = (r[2] >> 30) & 1; }
    if (max >= 7) { query(7, 0, r); seed = (r[1] >> 18) & 1; }
    query(0x80000000, 0, r);
    if (r[0] >= 0x80000001) { query(0x80000001, 0, r); tscp = (r[3] >> 27) & 1; }
    printf("CPUID TSC=%u RDTSCP=%u RDRAND=%u RDSEED=%u\n", tsc, tscp, rand, seed);
    machine_fences(); // SSE2 is part of the x86-64 ABI.
    uint64_t untouched = UINT64_C(0x123456789abcdef0);
    machine_prefetch(&untouched);
    if (untouched != UINT64_C(0x123456789abcdef0)) return 16;
    if (tsc) {
        int matched = 0;
        for (unsigned i = 0; i < 100; ++i) {
            uint64_t before = counter(), value = machine_tsc(), after = counter();
            if (before <= value && value <= after) { matched = 1; break; }
        }
        if (!matched) return 10;
    }
    if (tscp) {
        int matched = 0;
        for (unsigned i = 0; i < 100; ++i) {
            uint32_t a, b, actual;
            uint64_t before = counter_aux(&a), value = machine_tscp(&actual), after = counter_aux(&b);
            if (a == b && actual == a && before <= value && value <= after) { matched = 1; break; }
        }
        if (!matched) return 11;
    }
    if (rand) {
        uint64_t value;
        unsigned ok = oracle_random(&value, 0);
        if (ok > 1 || (!ok && value)) return 12;
        if (random_checks(machine_random16, 16) || random_checks(machine_random32, 32) || random_checks(machine_random64, 64)) return 13;
    }
    if (seed) {
        uint64_t value;
        unsigned ok = oracle_random(&value, 1);
        if (ok > 1 || (!ok && value)) return 14;
        if (random_checks(machine_seed16, 16) || random_checks(machine_seed32, 32) || random_checks(machine_seed64, 64)) return 15;
    }
    return 0;
}
