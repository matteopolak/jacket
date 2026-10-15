#include <cpuid.h>
#include <stdint.h>
#include <stdio.h>
#include <assert.h>
extern void cpu_query(uint32_t, uint32_t, uint64_t[4]);
int main(void) {
    const uint32_t cases[][2] = {{0,0},{7,0},{7,1},{0x80000000u,0}};
    for (unsigned i=0; i<sizeof(cases)/sizeof(cases[0]); ++i) {
        unsigned a,b,c,d;
        __cpuid_count(cases[i][0],cases[i][1],a,b,c,d);
        uint64_t output[4]={UINT64_MAX,UINT64_MAX,UINT64_MAX,UINT64_MAX};
        register uint64_t marker __asm__("rbx") = UINT64_C(0x1425364758697a8b);
        __asm__ volatile("" : "+r"(marker));
        cpu_query(cases[i][0],cases[i][1],output);
        __asm__ volatile("" : "+r"(marker));
        assert(marker==UINT64_C(0x1425364758697a8b));
        assert(output[0]==a && output[1]==b && output[2]==c && output[3]==d);
        if (i==0) assert((b|c|d)!=0);
    }
    puts("CPUID hardware registers, subleaves and callee-save ABI: ok");
    return 0;
}
