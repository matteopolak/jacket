/* Actual official native compiler C intrinsic, not a Jai/library implementation. */
#include <emmintrin.h>

int main(void) {
    for (int i = 0; i < 256; ++i)
        _mm_pause();
    return 0;
}
