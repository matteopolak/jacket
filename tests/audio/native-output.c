/* Real SDK operations with zero PCM. No synthetic device or success path. */
#define _POSIX_C_SOURCE 200809L
#include "output.h"
#include <stdio.h>
#include <string.h>
#define CHECK(condition) do { if (!(condition)) { fprintf(stderr, "SDK check failed at line %d\n", __LINE__); return 1; } } while (0)
#if defined(_WIN32)
#include <windows.h>
static void wait_ms(void) { Sleep(30); }
#else
#include <time.h>
static void wait_ms(void) { struct timespec duration = {0, 30000000}; nanosleep(&duration, NULL); }
#endif

int main(int argc, char **argv) {
    int required = argc == 2 && !strcmp(argv[1], "--require-device");
    int error = 0;
    CHECK(audio_output_clock() > 0);
    CHECK(audio_output_device(-1, (char[16]){0}, 16) < 0);
    char name[1024];
    for (int pass = 0; pass < 3; ++pass)
        for (int index = 0;; ++index) {
            int status = audio_output_device(index, name, sizeof(name));
            CHECK(status >= 0);
            if (!status) break;
            CHECK(name[0]);
        }
    for (int attempt = 0; attempt < 2; ++attempt) {
        AudioOutput *output = audio_output_open(44100, 2, &error);
        if (!output) {
            printf("native-device-unavailable (%d)\n", error);
            return required ? 1 : 0;
        }
        CHECK(audio_output_rate(output) == 44100 && audio_output_channels(output) == 2);
        CHECK(audio_output_capacity(output) > 0);
        CHECK(audio_output_submit(output, (char[3]){0}, 3) < 0);
        static const unsigned char silence[32768] = {0};
        int64_t capacity = audio_output_capacity(output);
        int count = capacity < (int64_t)sizeof(silence) ? (int)capacity : (int)sizeof(silence);
        count -= count % 4;
        CHECK(audio_output_submit(output, silence, count) == 0);
        CHECK(audio_output_play(output) == 0);
        int pause_status = audio_output_set_active(output, 0);
#if defined(_WIN32) || defined(__APPLE__)
        CHECK(pause_status == 0);
#endif
        if (!pause_status) {
            wait_ms();
            int64_t paused = audio_output_buffered(output);
            CHECK(paused >= 0);
            wait_ms();
            CHECK(audio_output_buffered(output) == paused);
            CHECK(audio_output_set_active(output, 1) == 0);
        } else printf("native-manual-pause-unsupported (%d)\n", pause_status);
        CHECK(audio_output_poll(output, 1) == 0);
        CHECK(audio_output_close(output) == 0);
    }
    puts("native-device-opened: SDK pause/resume, fresh discovery and controlled reopen passed");
    return 0;
}
