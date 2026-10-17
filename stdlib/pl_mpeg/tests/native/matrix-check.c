/* Licensed public C algorithm comparison, not a Jai API oracle. */
#include <stddef.h>
#include <stdio.h>
#define PL_MPEG_IMPLEMENTATION
#include "../../../../third_party/pl_mpeg/pl_mpeg.h"

int main(void) {
    for (int basis = 0; basis < 32; ++basis) {
        int samples[32][3] = {{0}};
        float result[64];
        samples[basis][0] = 1;
        plm_audio_idct36(samples, 0, result, 0);
        if (fwrite(result, sizeof(float), 64, stdout) != 64) return 1;
    }
    return fflush(stdout) == 0 ? 0 : 1;
}
