#include <cpuid.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    unsigned int a, b, c, d;
    if (!__get_cpuid(0, &a, &b, &c, &d)) {
        fputs("native CPUID query unavailable\n", stderr);
        return 1;
    }
    if ((b | c | d) == 0) {
        fputs("native CPUID returned empty vendor\n", stderr);
        return 2;
    }
    char vendor[13];
    memcpy(vendor, &b, 4);
    memcpy(vendor + 4, &d, 4);
    memcpy(vendor + 8, &c, 4);
    vendor[12] = '\0';
    printf("native CPUID vendor=%s max_leaf=%u\n", vendor, a);
    return 0;
}
