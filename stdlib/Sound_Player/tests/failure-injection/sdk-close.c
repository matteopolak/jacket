/* Inject an error at a public SDK close boundary while owning a real device.
   This tests wrapper ownership; it does not claim an OS-generated failure. */
#define _POSIX_C_SOURCE 200809L
static int armed, close_calls;
#if defined(_WIN32)
#include <windows.h>
#include <mmsystem.h>
static MMRESULT WINAPI controlled_reset(HWAVEOUT device);
#define waveOutReset controlled_reset
#elif defined(__APPLE__)
#include <AudioToolbox/AudioToolbox.h>
static OSStatus controlled_dispose(AudioQueueRef queue, Boolean immediate);
#define AudioQueueDispose controlled_dispose
#else
#include <alsa/asoundlib.h>
static int controlled_close(snd_pcm_t *pcm);
#define snd_pcm_close controlled_close
#endif
#include "../../../internal/audio/native/output.c"
#if defined(_WIN32)
#undef waveOutReset
static MMRESULT WINAPI controlled_reset(HWAVEOUT device) {
    ++close_calls;
    if (armed) { armed = 0; return MMSYSERR_ERROR; }
    return waveOutReset(device);
}
#elif defined(__APPLE__)
#undef AudioQueueDispose
static OSStatus controlled_dispose(AudioQueueRef queue, Boolean immediate) {
    ++close_calls;
    if (armed) { armed = 0; return -777; }
    return AudioQueueDispose(queue, immediate);
}
#else
#undef snd_pcm_close
static int controlled_close(snd_pcm_t *pcm) {
    ++close_calls;
    int result = snd_pcm_close(pcm);
    if (armed) { armed = 0; return -777; }
    return result;
}
#endif

#define CHECK(condition) do { if (!(condition)) { fprintf(stderr, "SDK close check failed at line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    int error;
    AudioOutput *output = audio_output_open(44100, 2, &error);
    CHECK(output != NULL);
    static const char silence[4096] = {0};
    CHECK(audio_output_submit(output, silence, sizeof(silence)) == 0);
    CHECK(audio_output_play(output) == 0);
    armed = 1;
    CHECK(audio_output_close(output) != 0);
    CHECK(close_calls == 1 && audio_output_rate(output) == 44100);
#if !defined(_WIN32) && !defined(__APPLE__)
    CHECK(output->pcm == NULL); /* ALSA's error return still consumes its PCM. */
#endif
    CHECK(audio_output_close(output) == 0);
#if defined(_WIN32) || defined(__APPLE__)
    CHECK(close_calls == 2);
#else
    CHECK(close_calls == 1);
#endif
    puts("ok");
    return 0;
}
