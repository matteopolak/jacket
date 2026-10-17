// Original public C intrinsic oracle; no optional or privileged instruction.
#include <stdint.h>
#include <stdio.h>
#if defined(_MSC_VER)
#include <intrin.h>
#else
#include <cpuid.h>
#endif

extern uint32_t cpuid_register(uint32_t leaf, uint32_t subleaf, uint32_t index);

int main(void) {
    uint32_t expected[4];
#if defined(_MSC_VER)
    int registers[4];
    __cpuidex(registers, 0, 0);
    for (unsigned i = 0; i < 4; ++i) expected[i] = (uint32_t)registers[i];
#else
    __cpuid_count(0, 0, expected[0], expected[1], expected[2], expected[3]);
#endif
    if (expected[0] < 1 || !(expected[1] | expected[2] | expected[3])) return 1;
    for (unsigned i = 0; i < 4; ++i) {
        uint32_t actual = cpuid_register(0, 0, i);
        if (actual != expected[i]) {
            fprintf(stderr, "leaf zero register %u: got %08x expected %08x\n",
                    i, actual, expected[i]);
            return 2;
        }
    }
    puts("CPUID leaf zero: all four fields match the host intrinsic");
    return 0;
}
