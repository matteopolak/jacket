/* Test-only boundary failure: open/submit/play/close otherwise use the real SDK.
   No synthetic handle or successful device operation is supplied. */
#define audio_output_close audio_test_real_close
#define audio_output_play audio_test_real_play
#include "../../../stdlib/internal/audio/native/output.c"
#undef audio_output_close
#undef audio_output_play

static int fail_close;
static int fail_play;
JAI_AUDIO_API void audio_test_fail_next_close(void) { fail_close = 1; }
JAI_AUDIO_API void audio_test_fail_next_play(void) { fail_play = 1; }
JAI_AUDIO_API int audio_output_play(AudioOutput *output) {
    if (fail_play && output) { fail_play = 0; return -778; }
    return audio_test_real_play(output);
}
JAI_AUDIO_API int audio_output_close(AudioOutput *output) {
    if (fail_close && output) {
        fail_close = 0;
        return -777;
    }
    return audio_test_real_close(output);
}
