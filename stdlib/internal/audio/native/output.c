/* Independently authored output ownership adapter against public OS audio SDKs.
 * No Jai callback enters a native audio thread. The producer is serialized by
 * Sound_Player; OS callbacks only publish slot availability and byte counts. */
#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <time.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdio.h>
#define JAI_AUDIO_BUILD 1
#include "output.h"
#include "routing.h"

#if defined(__ANDROID__)
#error Android requires a distinct AAudio backend; it is not an ALSA target
#elif !defined(_WIN32) && !defined(__APPLE__) && !defined(__linux__)
#error Unsupported native audio SDK target
#endif

#if defined(_WIN32)
#include <windows.h>
#include <mmsystem.h>
#include <mmreg.h>
#elif defined(__APPLE__)
#include <AudioToolbox/AudioToolbox.h>
#include <CoreAudio/CoreAudio.h>
#else
#include <alsa/asoundlib.h>
#include <errno.h>
#endif

enum { SLOT_COUNT = 8, SLOT_BYTES = 96000 };
struct AudioOutput {
    int rate, channels, started, paused, last_error;
    _Atomic int64_t queued;
    unsigned channel_order[8];
#if defined(_WIN32)
    HWAVEOUT device;
    WAVEHDR buffers[SLOT_COUNT];
    uint8_t storage[SLOT_COUNT][SLOT_BYTES];
#elif defined(__APPLE__)
    AudioQueueRef queue;
    AudioQueueBufferRef buffers[SLOT_COUNT];
    _Atomic int occupied[SLOT_COUNT];
    AudioDeviceID default_device;
#else
    snd_pcm_t *pcm;
    uint8_t mapped[SLOT_BYTES];
#endif
};

#if defined(__APPLE__)
extern int audio_output_process_active(void);
#elif defined(_WIN32)
static BOOL CALLBACK visible_process_window(HWND window, LPARAM opaque) {
    DWORD process = 0;
    GetWindowThreadProcessId(window, &process);
    if (process == GetCurrentProcessId() && IsWindowVisible(window) && !GetWindow(window, GW_OWNER)) {
        *(int *)opaque = 1;
        return FALSE;
    }
    return TRUE;
}
JAI_AUDIO_API int audio_output_process_active(void) {
    static _Atomic int has_owned_window;
    int owns_window = 0;
    EnumWindows(visible_process_window, (LPARAM)&owns_window);
    if (owns_window) atomic_store(&has_owned_window, 1);
    if (!atomic_load(&has_owned_window)) return 1; /* CLI/headless process has no window focus. */
    DWORD process = 0;
    HWND window = GetForegroundWindow();
    if (window) GetWindowThreadProcessId(window, &process);
    return process == GetCurrentProcessId();
}
#else
JAI_AUDIO_API int audio_output_process_active(void) { return 1; } /* ALSA has no window API. */
#endif

JAI_AUDIO_API int audio_output_last_error(AudioOutput *output) { return output ? output->last_error : -1; }

JAI_AUDIO_API int audio_output_set_active(AudioOutput *output, int active) {
    if (!output) return -1;
    int paused = !active;
    if (output->paused == paused) return 0;
    int result = 0;
#if defined(__APPLE__)
    result = (int)(paused ? AudioQueuePause(output->queue) : AudioQueueStart(output->queue, NULL));
#elif defined(_WIN32)
    result = (int)(paused ? waveOutPause(output->device) : waveOutRestart(output->device));
#else
    result = snd_pcm_pause(output->pcm, paused);
#endif
    if (!result) output->paused = paused;
    else output->last_error = result;
    return result;
}

/* 0 ready, 1 intentionally paused/retry later, 2 device needs reopening.
 * Called by the serialized producer, never by an OS callback. */
JAI_AUDIO_API int audio_output_poll(AudioOutput *output, int allow_background) {
    if (!output) return 2;
    if (audio_output_set_active(output, allow_background || audio_output_process_active())) return 2;
#if defined(__APPLE__)
    AudioObjectPropertyAddress address = {kAudioHardwarePropertyDefaultOutputDevice,
                                         kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain};
    AudioDeviceID device = kAudioObjectUnknown;
    UInt32 size = sizeof(device);
    int result = (int)AudioObjectGetPropertyData(kAudioObjectSystemObject, &address, 0, NULL, &size, &device);
    if (result || device == kAudioObjectUnknown || device != output->default_device) {
        output->last_error = result ? result : -3;
        return 2;
    }
    if (output->started && !output->paused && atomic_load(&output->queued) > 0) {
        UInt32 running = 0;
        size = sizeof(running);
        result = (int)AudioQueueGetProperty(output->queue, kAudioQueueProperty_IsRunning, &running, &size);
        if (!result && !running) result = (int)AudioQueueStart(output->queue, NULL);
        if (result) { output->last_error = result; return 2; }
    }
#elif !defined(_WIN32)
    snd_pcm_state_t state = snd_pcm_state(output->pcm);
    int result = 0;
    if (state == SND_PCM_STATE_DISCONNECTED) { output->last_error = -ENODEV; return 2; }
    if (state == SND_PCM_STATE_SUSPENDED) {
        result = snd_pcm_resume(output->pcm);
        if (result == -EAGAIN) return 1;
        if (result < 0) result = snd_pcm_prepare(output->pcm);
    } else if (state == SND_PCM_STATE_XRUN) result = snd_pcm_prepare(output->pcm);
    if (result < 0) { output->last_error = result; return 2; }
#endif
    return output->paused ? 1 : 0;
}

JAI_AUDIO_API int audio_output_rate(AudioOutput *output) { return output ? output->rate : 0; }
JAI_AUDIO_API int audio_output_channels(AudioOutput *output) { return output ? output->channels : 0; }
JAI_AUDIO_API int64_t audio_output_capacity(AudioOutput *output) {
    if (!output) return -1;
#if defined(_WIN32) || defined(__APPLE__)
    return (int64_t)SLOT_COUNT * SLOT_BYTES;
#else
    snd_pcm_uframes_t frames = 0, period = 0;
    int result = snd_pcm_get_params(output->pcm, &frames, &period);
    return result < 0 ? result : (int64_t)frames * output->channels * 2;
#endif
}

JAI_AUDIO_API int audio_output_preferred_channels(void) {
#if defined(_WIN32)
    /* UCRT supplies a private copy. An acquisition error must not silently
       select the default device configuration. */
    char *requested = NULL;
    if (_dupenv_s(&requested, NULL, "JAI_AUDIO_CHANNELS")) return -1;
#else
    const char *requested = getenv("JAI_AUDIO_CHANNELS");
#endif
    int channels_requested = 0;
    if (requested && *requested) {
        channels_requested = !strcmp(requested, "2") ? 2 :
                             !strcmp(requested, "6") ? 6 :
                             !strcmp(requested, "8") ? 8 : -1;
    }
#if defined(_WIN32)
    free(requested);
#endif
    if (channels_requested) return channels_requested;
#if defined(__APPLE__)
    AudioObjectPropertyAddress address = {kAudioHardwarePropertyDefaultOutputDevice,
                                         kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain};
    AudioDeviceID device = kAudioObjectUnknown;
    UInt32 size = sizeof(device);
    if (AudioObjectGetPropertyData(kAudioObjectSystemObject, &address, 0, NULL, &size, &device)) return 2;
    address = (AudioObjectPropertyAddress){kAudioDevicePropertyStreamConfiguration,
                                          kAudioDevicePropertyScopeOutput, kAudioObjectPropertyElementMain};
    if (AudioObjectGetPropertyDataSize(device, &address, 0, NULL, &size)) return 2;
    AudioBufferList *buffers = malloc(size ? size : 1);
    if (!buffers) return -1;
    int channels = 0;
    if (!AudioObjectGetPropertyData(device, &address, 0, NULL, &size, buffers))
        for (UInt32 i = 0; i < buffers->mNumberBuffers; ++i) channels += (int)buffers->mBuffers[i].mNumberChannels;
    free(buffers);
    if (channels == 6 || channels == 8) return channels;
#elif defined(_WIN32)
    WAVEOUTCAPSW caps;
    if (!waveOutGetDevCapsW((UINT_PTR)WAVE_MAPPER, &caps, sizeof(caps))) {
        if (caps.wChannels == 6 || caps.wChannels == 8) return caps.wChannels;
    }
#endif
    return 2;
}

JAI_AUDIO_API double audio_output_clock(void) {
#if defined(_WIN32)
    LARGE_INTEGER ticks, frequency;
    if (!QueryPerformanceFrequency(&frequency) || !QueryPerformanceCounter(&ticks)) return -1;
    return (double)ticks.QuadPart / (double)frequency.QuadPart;
#else
    struct timespec time;
    if (clock_gettime(CLOCK_MONOTONIC, &time)) return -1;
    return (double)time.tv_sec + (double)time.tv_nsec / 1000000000.0;
#endif
}

#if defined(__APPLE__)
static void completed(void *opaque, AudioQueueRef queue, AudioQueueBufferRef buffer) {
    (void)queue;
    AudioOutput *output = opaque;
    for (int i = 0; i < SLOT_COUNT; ++i) {
        if (output->buffers[i] == buffer) {
            atomic_fetch_sub(&output->queued, buffer->mAudioDataByteSize);
            atomic_store(&output->occupied[i], 0);
            return;
        }
    }
}
#endif

JAI_AUDIO_API int audio_output_close(AudioOutput *output) {
    if (!output) return 0;
    int error = 0;
#if defined(_WIN32)
    if (!output->device) { free(output); return 0; }
    error = (int)waveOutReset(output->device);
    if (error) return error;
    for (int i = 0; i < SLOT_COUNT; ++i) {
        if (output->buffers[i].dwFlags & WHDR_PREPARED) {
            int result = (int)waveOutUnprepareHeader(output->device, &output->buffers[i], sizeof(WAVEHDR));
            if (result) return result;
        }
    }
    int result = (int)waveOutClose(output->device);
    if (result) return result; /* Native headers can still refer to our buffers. */
#elif defined(__APPLE__)
    if (output->queue) error = (int)AudioQueueDispose(output->queue, true);
    if (error) return error; /* Do not free callback state after failed disposal. */
#else
    if (output->pcm) {
        error = snd_pcm_close(output->pcm);
        /* ALSA consumes the PCM even when cleanup reports an error. Retain
           only our wrapper for retry; never reuse a consumed PCM pointer. */
        output->pcm = NULL;
    }
    if (error) return error;
#endif
    free(output);
    return error;
}

JAI_AUDIO_API AudioOutput *audio_output_open(int rate, int channels, int *error) {
    *error = 0;
    if (rate <= 0 || channels <= 0 || channels > 8) { *error = -1; return NULL; }
    AudioOutput *output = calloc(1, sizeof(*output));
    if (!output) { *error = -2; return NULL; }
    output->rate = rate;
    output->channels = channels;
    atomic_init(&output->queued, 0);
    for (int c = 0; c < channels; ++c) output->channel_order[c] = (unsigned)c;
#if defined(__APPLE__)
    for (int i = 0; i < SLOT_COUNT; ++i) atomic_init(&output->occupied[i], 0);
#endif
#if defined(_WIN32)
    WAVEFORMATEXTENSIBLE format = {0};
    format.Format.wFormatTag = channels > 2 ? WAVE_FORMAT_EXTENSIBLE : WAVE_FORMAT_PCM;
    format.Format.nChannels = (WORD)channels;
    format.Format.nSamplesPerSec = (DWORD)rate;
    format.Format.wBitsPerSample = 16;
    format.Format.nBlockAlign = (WORD)(channels * 2);
    format.Format.nAvgBytesPerSec = (DWORD)(rate * channels * 2);
    if (channels > 2) {
        format.Format.cbSize = 22;
        format.Samples.wValidBitsPerSample = 16;
        format.dwChannelMask = channels == 6 ? 0x3f : 0x63f;
        format.SubFormat = (GUID){1, 0, 0x10, {0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71}};
    }
    *error = (int)waveOutOpen(&output->device, WAVE_MAPPER, &format.Format, 0, 0, CALLBACK_NULL);
    if (!*error) *error = (int)waveOutPause(output->device);
#elif defined(__APPLE__)
    AudioObjectPropertyAddress address = {kAudioHardwarePropertyDefaultOutputDevice,
                                         kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain};
    AudioDeviceID device = kAudioObjectUnknown;
    UInt32 size = sizeof(device);
    *error = (int)AudioObjectGetPropertyData(kAudioObjectSystemObject, &address, 0, NULL, &size, &device);
    if (!*error && device == kAudioObjectUnknown) *error = -3;
    output->default_device = device;
    AudioStreamBasicDescription format = {0};
    format.mSampleRate = rate;
    format.mFormatID = kAudioFormatLinearPCM;
    format.mFormatFlags = kLinearPCMFormatFlagIsSignedInteger | kLinearPCMFormatFlagIsPacked;
    format.mBytesPerPacket = (UInt32)(channels * 2);
    format.mFramesPerPacket = 1;
    format.mBytesPerFrame = (UInt32)(channels * 2);
    format.mChannelsPerFrame = (UInt32)channels;
    format.mBitsPerChannel = 16;
    if (!*error) *error = (int)AudioQueueNewOutput(&format, completed, output, NULL, NULL, 0, &output->queue);
    if (!*error && channels > 2) {
        UInt32 layout_size = (UInt32)(offsetof(AudioChannelLayout, mChannelDescriptions)
                                     + (size_t)channels * sizeof(AudioChannelDescription));
        AudioChannelLayout *layout = calloc(1, layout_size);
        if (!layout) *error = -2;
        else {
            const AudioChannelLabel labels[8] = {kAudioChannelLabel_Left, kAudioChannelLabel_Right,
                kAudioChannelLabel_Center, kAudioChannelLabel_LFEScreen,
                channels == 8 ? kAudioChannelLabel_RearSurroundLeft : kAudioChannelLabel_LeftSurround,
                channels == 8 ? kAudioChannelLabel_RearSurroundRight : kAudioChannelLabel_RightSurround,
                kAudioChannelLabel_LeftSurround, kAudioChannelLabel_RightSurround};
            layout->mChannelLayoutTag = kAudioChannelLayoutTag_UseChannelDescriptions;
            layout->mNumberChannelDescriptions = (UInt32)channels;
            for (int i = 0; i < channels; ++i) layout->mChannelDescriptions[i].mChannelLabel = labels[i];
            *error = (int)AudioQueueSetProperty(output->queue, kAudioQueueProperty_ChannelLayout, layout, layout_size);
            free(layout);
        }
    }
    for (int i = 0; !*error && i < SLOT_COUNT; ++i)
        *error = (int)AudioQueueAllocateBuffer(output->queue, SLOT_BYTES, &output->buffers[i]);
#else
    const char *device = getenv("JAI_AUDIO_ALSA_DEVICE");
    if (!device || !*device) device = "default";
    *error = snd_pcm_open(&output->pcm, device, SND_PCM_STREAM_PLAYBACK, SND_PCM_NONBLOCK);
    if (!*error) *error = snd_pcm_set_params(output->pcm, SND_PCM_FORMAT_S16_LE,
                                            SND_PCM_ACCESS_RW_INTERLEAVED, (unsigned)channels,
                                            (unsigned)rate, 1, 100000);
    if (!*error) {
        snd_pcm_hw_params_t *parameters = NULL;
        unsigned actual_rate = 0, actual_channels = 0;
        int direction = 0;
        *error = snd_pcm_hw_params_malloc(&parameters);
        if (!*error) *error = snd_pcm_hw_params_current(output->pcm, parameters);
        if (!*error) *error = snd_pcm_hw_params_get_rate(parameters, &actual_rate, &direction);
        if (!*error) *error = snd_pcm_hw_params_get_channels(parameters, &actual_channels);
        if (!*error && (!actual_rate || actual_rate > INT32_MAX || actual_channels != (unsigned)channels)) *error = -EINVAL;
        if (!*error) output->rate = (int)actual_rate;
        if (parameters) snd_pcm_hw_params_free(parameters);
    }
    if (!*error) {
        snd_pcm_chmap_t *map = snd_pcm_get_chmap(output->pcm);
        if (!map) {
            /* A multichannel order must be known, never guessed from count. */
            if (channels > 2) *error = -ENOTSUP;
        } else {
            if (map->channels != (unsigned)channels) *error = -EINVAL;
            for (int c = 0; !*error && c < channels; ++c) {
                unsigned logical;
                switch (map->pos[c]) {
                    case SND_CHMAP_FL: logical = 0; break;
                    case SND_CHMAP_FR: logical = 1; break;
                    case SND_CHMAP_FC: logical = 2; break;
                    case SND_CHMAP_LFE: logical = 3; break;
                    case SND_CHMAP_RL: logical = 4; break;
                    case SND_CHMAP_RR: logical = 5; break;
                    case SND_CHMAP_SL: logical = channels == 8 ? 6 : 4; break;
                    case SND_CHMAP_SR: logical = channels == 8 ? 7 : 5; break;
                    default: *error = -ENOTSUP; logical = 0; break;
                }
                output->channel_order[c] = logical;
            }
            if (!*error && !audio_valid_channel_order(output->channel_order, channels)) *error = -EINVAL;
            free(map);
        }
    }
#endif
    if (*error) {
        int cleanup = audio_output_close(output);
        if (cleanup) {
            *error = cleanup;
            fprintf(stderr, "Sound_Player: native initialization cleanup failed (%d)\n", cleanup);
            abort(); /* Cannot abandon state that a native callback may retain. */
        }
        return NULL;
    }
    return output;
}

JAI_AUDIO_API int64_t audio_output_buffered(AudioOutput *output) {
    if (!output) return -1;
#if defined(_WIN32)
    int64_t bytes = 0;
    for (int i = 0; i < SLOT_COUNT; ++i) {
        WAVEHDR *buffer = &output->buffers[i];
        if ((buffer->dwFlags & WHDR_PREPARED) && !(buffer->dwFlags & WHDR_DONE)) bytes += buffer->dwBufferLength;
    }
    return bytes;
#elif defined(__APPLE__)
    return atomic_load(&output->queued);
#else
    snd_pcm_sframes_t frames = 0;
    int result = snd_pcm_delay(output->pcm, &frames);
    if (result < 0) return result;
    return frames > 0 ? frames * output->channels * 2 : 0;
#endif
}

JAI_AUDIO_API int audio_output_submit(AudioOutput *output, const void *bytes, int count) {
    if (!output || !bytes || count <= 0 || count > SLOT_BYTES || count % (output->channels * 2)) return -1;
#if defined(_WIN32)
    for (int i = 0; i < SLOT_COUNT; ++i) {
        WAVEHDR *buffer = &output->buffers[i];
        if ((buffer->dwFlags & WHDR_PREPARED) && !(buffer->dwFlags & WHDR_DONE)) continue;
        if (buffer->dwFlags & WHDR_PREPARED) {
            int result = (int)waveOutUnprepareHeader(output->device, buffer, sizeof(*buffer));
            if (result) return result;
        }
        memset(buffer, 0, sizeof(*buffer));
        memcpy(output->storage[i], bytes, (size_t)count);
        buffer->lpData = (LPSTR)output->storage[i];
        buffer->dwBufferLength = (DWORD)count;
        int result = (int)waveOutPrepareHeader(output->device, buffer, sizeof(*buffer));
        if (result) return result;
        return (int)waveOutWrite(output->device, buffer, sizeof(*buffer));
    }
    return -2;
#elif defined(__APPLE__)
    for (int i = 0; i < SLOT_COUNT; ++i) {
        if (atomic_exchange(&output->occupied[i], 1)) continue;
        AudioQueueBufferRef buffer = output->buffers[i];
        memcpy(buffer->mAudioData, bytes, (size_t)count);
        buffer->mAudioDataByteSize = (UInt32)count;
        atomic_fetch_add(&output->queued, count);
        int result = (int)AudioQueueEnqueueBuffer(output->queue, buffer, 0, NULL);
        if (result) {
            atomic_fetch_sub(&output->queued, count);
            atomic_store(&output->occupied[i], 0);
        }
        return result;
    }
    return -2;
#else
    const uint8_t *cursor = bytes;
    int reordered = 0;
    for (int c = 0; c < output->channels; ++c)
        if (output->channel_order[c] != (unsigned)c) reordered = 1;
    if (reordered) {
        audio_copy_channels(output->mapped, bytes, (size_t)count, output->channels, output->channel_order);
        cursor = output->mapped;
    }
    snd_pcm_sframes_t remaining = count / (output->channels * 2);
    int retries = 0;
    while (remaining > 0) {
        snd_pcm_sframes_t written = snd_pcm_writei(output->pcm, cursor, (snd_pcm_uframes_t)remaining);
        if (written < 0) {
            if (++retries > 8) return (int)written;
            if (written == -EINTR) continue;
            if (written != -EPIPE && written != -ESTRPIPE) return (int)written;
            // Resume is nonblocking: a suspended device can report EAGAIN and
            // enter the producer's bounded reopen path, never a sleep loop.
            int recovery = written == -EPIPE ? snd_pcm_prepare(output->pcm) : snd_pcm_resume(output->pcm);
            if (recovery < 0) return recovery;
            continue;
        }
        if (!written) return -EAGAIN;
        cursor += written * output->channels * 2;
        remaining -= written;
    }
    if (output->started && !output->paused && snd_pcm_state(output->pcm) == SND_PCM_STATE_PREPARED)
        return snd_pcm_start(output->pcm);
    return 0;
#endif
}

JAI_AUDIO_API int audio_output_play(AudioOutput *output) {
    if (!output) return -1;
    if (output->started) return 0;
    if (output->paused) { output->started = 1; return 0; }
    int result = 0;
#if defined(__APPLE__)
    result = (int)AudioQueueStart(output->queue, NULL);
#elif defined(_WIN32)
    result = (int)waveOutRestart(output->device);
#else
    if (snd_pcm_state(output->pcm) != SND_PCM_STATE_RUNNING) result = snd_pcm_start(output->pcm);
#endif
    if (!result) output->started = 1;
    return result;
}

/* Returns real device names, or zero at end. No invented fallback device. */
JAI_AUDIO_API int audio_output_device(int index, char *name, int capacity) {
    if (index < 0 || capacity < 2) return -1;
    name[0] = 0;
#if defined(_WIN32)
    if ((UINT)index >= waveOutGetNumDevs()) return 0;
    WAVEOUTCAPSW caps;
    int result = (int)waveOutGetDevCapsW((UINT_PTR)index, &caps, sizeof(caps));
    if (result) return -result;
    int converted = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, caps.szPname, -1,
                                        name, capacity, NULL, NULL);
    return converted > 0 ? 1 : -1;
#elif defined(__APPLE__)
    AudioObjectPropertyAddress address = {kAudioHardwarePropertyDevices,
                                         kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain};
    UInt32 size = 0;
    if (AudioObjectGetPropertyDataSize(kAudioObjectSystemObject, &address, 0, NULL, &size)) return -1;
    AudioDeviceID *devices = malloc(size ? size : 1);
    if (!devices) return -1;
    if (AudioObjectGetPropertyData(kAudioObjectSystemObject, &address, 0, NULL, &size, devices)) { free(devices); return -1; }
    int selected = 0;
    for (UInt32 i = 0; i < size / sizeof(*devices); ++i) {
        AudioObjectPropertyAddress streams = {kAudioDevicePropertyStreams, kAudioDevicePropertyScopeOutput, kAudioObjectPropertyElementMain};
        UInt32 stream_size = 0;
        if (AudioObjectGetPropertyDataSize(devices[i], &streams, 0, NULL, &stream_size) || !stream_size) continue;
        if (selected++ != index) continue;
        AudioObjectPropertyAddress property = {kAudioObjectPropertyName, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain};
        CFStringRef label = NULL;
        UInt32 label_size = sizeof(label);
        OSStatus result = AudioObjectGetPropertyData(devices[i], &property, 0, NULL, &label_size, &label);
        free(devices);
        if (result || !label) return -1;
        Boolean copied = CFStringGetCString(label, name, capacity, kCFStringEncodingUTF8);
        CFRelease(label);
        return copied ? 1 : -1;
    }
    free(devices);
    return 0;
#else
    void **hints = NULL;
    if (snd_device_name_hint(-1, "pcm", &hints) < 0) return -1;
    int selected = 0, result = 0;
    for (void **hint = hints; *hint; ++hint) {
        char *io = snd_device_name_get_hint(*hint, "IOID");
        int input_only = io && !strcmp(io, "Input");
        free(io);
        if (input_only || selected++ != index) continue;
        char *label = snd_device_name_get_hint(*hint, "NAME");
        if (!label) { result = -1; break; }
        size_t length = strlen(label);
        if (length >= (size_t)capacity) result = -1;
        else { memcpy(name, label, length + 1); result = 1; }
        free(label);
        break;
    }
    int cleanup = snd_device_name_free_hint(hints);
    return cleanup < 0 ? cleanup : result;
#endif
}
