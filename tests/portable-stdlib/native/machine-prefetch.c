/* Official native C intrinsic instruction probe for the compiler handoff. */
#include <xmmintrin.h>

int main(void) {
    char data = 73;
    _mm_prefetch(&data, _MM_HINT_T0);
    return 0;
}
