#ifndef JAI_AUDIO_OUTPUT_H
#define JAI_AUDIO_OUTPUT_H
#include <stdint.h>
#if defined(_WIN32) && defined(JAI_AUDIO_BUILD)
#define JAI_AUDIO_API __declspec(dllexport)
#elif defined(_WIN32)
#define JAI_AUDIO_API
#else
#define JAI_AUDIO_API __attribute__((visibility("default")))
#endif
typedef struct AudioOutput AudioOutput;
JAI_AUDIO_API AudioOutput *audio_output_open(int rate, int channels, int *error);
JAI_AUDIO_API int audio_output_close(AudioOutput *output);
JAI_AUDIO_API int audio_output_rate(AudioOutput *output);
JAI_AUDIO_API int audio_output_channels(AudioOutput *output);
JAI_AUDIO_API int64_t audio_output_capacity(AudioOutput *output);
JAI_AUDIO_API int64_t audio_output_buffered(AudioOutput *output);
JAI_AUDIO_API int audio_output_submit(AudioOutput *output, const void *bytes, int count);
JAI_AUDIO_API int audio_output_play(AudioOutput *output);
JAI_AUDIO_API double audio_output_clock(void);
JAI_AUDIO_API int audio_output_device(int index, char *name, int capacity);
JAI_AUDIO_API int audio_output_preferred_channels(void);
JAI_AUDIO_API int audio_output_process_active(void);
JAI_AUDIO_API int audio_output_set_active(AudioOutput *output, int active);
JAI_AUDIO_API int audio_output_poll(AudioOutput *output, int allow_background);
JAI_AUDIO_API int audio_output_last_error(AudioOutput *output);
#endif
