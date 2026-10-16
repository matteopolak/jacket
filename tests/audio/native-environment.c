/* Producer-owned backend configuration contract, independent of device presence.
   Uses real CRT/POSIX environment APIs; no provider or device is fabricated. */
#define _POSIX_C_SOURCE 200809L
#include <stdlib.h>
#include <errno.h>
#if defined(_WIN32)
static int fail_read;
static errno_t controlled_dupenv_s(char **value, size_t *length, const char *name) {
    if (fail_read) {
        *value = NULL;
        if (length) *length = 0;
        return ENOMEM;
    }
    return _dupenv_s(value, length, name);
}
#define _dupenv_s controlled_dupenv_s
#endif
#include "../../stdlib/internal/audio/native/output.c"
#if defined(_WIN32)
#undef _dupenv_s
#endif
#define CHECK(condition) do { if (!(condition)) { fprintf(stderr, "environment check failed at line %d\n", __LINE__); return 1; } } while (0)
static int configure(const char *value) {
#if defined(_WIN32)
    return _putenv_s("JAI_AUDIO_CHANNELS", value ? value : "");
#else
    return value ? setenv("JAI_AUDIO_CHANNELS", value, 1) : unsetenv("JAI_AUDIO_CHANNELS");
#endif
}
int main(void) {
    CHECK(!configure(NULL));
    int discovered = audio_output_preferred_channels();
    CHECK(discovered == 2 || discovered == 6 || discovered == 8);
    CHECK(!configure(""));
    CHECK(audio_output_preferred_channels() == discovered);
    const char *values[] = {"2", "6", "8", "4", " 2", "2x", "-1", "999999999999999999999"};
    int expected[] = {2, 6, 8, -1, -1, -1, -1, -1};
    for (int repeat = 0; repeat < 1000; ++repeat) {
        for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); ++i) {
            CHECK(!configure(values[i]));
            CHECK(audio_output_preferred_channels() == expected[i]);
        }
    }
#if defined(_WIN32)
    fail_read = 1;
    CHECK(audio_output_preferred_channels() == -1);
    fail_read = 0;
    CHECK(!configure("6"));
    CHECK(audio_output_preferred_channels() == 6);
#endif
    CHECK(!configure(NULL));
    puts("ok");
    return 0;
}
