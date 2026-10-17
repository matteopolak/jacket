/* Independent official-native-header oracle; no Jai implementation input. */
#include <cpuid.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    unsigned a, b, c, d;
    uint32_t words[12] = {0};
    unsigned maximum = __get_cpuid_max(0, NULL);
    __cpuid_count(0, 0, a, b, c, d);
    char vendor[12];
    memcpy(vendor, &b, 4);
    memcpy(vendor + 4, &d, 4);
    memcpy(vendor + 8, &c, 4);
    if (memcmp(vendor, "GenuineIntel", 12) == 0) words[0] = 1;
    if (memcmp(vendor, "AuthenticAMD", 12) == 0) words[0] = 2;
    if (maximum >= 1) {
        __cpuid_count(1, 0, a, b, c, d);
        words[1] = d;
        words[2] = c;
    }
    if (maximum >= 7) {
        __cpuid_count(7, 0, a, b, c, d);
        unsigned subleaf_max = a;
        words[7] = b;
        words[8] = c;
        words[9] = d;
        if (subleaf_max >= 1) {
            __cpuid_count(7, 1, a, b, c, d);
            words[10] = a;
        }
    }
    if (maximum >= 13) {
        __cpuid_count(13, 1, a, b, c, d);
        words[11] = a;
    }
    if (maximum >= 25) {
        __cpuid_count(25, 0, a, b, c, d);
        words[3] = b;
    }
    unsigned extended = __get_cpuid_max(0x80000000, NULL);
    if (extended >= 0x80000001) {
        __cpuid_count(0x80000001, 0, a, b, c, d);
        words[4] = d;
        words[5] = c;
    }
    if (extended >= 0x80000008) {
        __cpuid_count(0x80000008, 0, a, b, c, d);
        words[6] = b;
    }
    for (unsigned i = 0; i < 12; ++i) {
        for (unsigned byte = 0; byte < 4; ++byte) {
            if (putchar((int)((words[i] >> (byte * 8)) & 255)) == EOF) return 1;
        }
    }
    return fflush(stdout) == 0 ? 0 : 1;
}
