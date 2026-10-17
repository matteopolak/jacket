/* Official native C intrinsic counterparts for the compiler-owned handoff. */
#include <emmintrin.h>

int main(void) {
    _mm_mfence();
    _mm_lfence();
    _mm_sfence();
    return 0;
}
