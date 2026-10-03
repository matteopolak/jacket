#ifndef JAI_AUDIO_ROUTING_H
#define JAI_AUDIO_ROUTING_H
#include <stdint.h>
#include <stddef.h>
#include <string.h>

static inline int audio_valid_channel_order(const unsigned *mapping, int channels) {
    unsigned seen = 0;
    for (int c = 0; c < channels; ++c) {
        if (mapping[c] >= (unsigned)channels || (seen & (1u << mapping[c]))) return 0;
        seen |= 1u << mapping[c];
    }
    return 1;
}

/* Each destination SDK slot names its source logical Sound_Player slot.
   Byte copies also support borrowed PCM with arbitrary alignment. */
static inline void audio_copy_channels(uint8_t *destination, const uint8_t *source,
                                      size_t bytes, int channels, const unsigned *mapping) {
    int identity = 1;
    for (int c = 0; c < channels; ++c) if (mapping[c] != (unsigned)c) identity = 0;
    if (identity) { memcpy(destination, source, bytes); return; }
    for (size_t frame = 0; frame < bytes; frame += (size_t)channels * 2)
        for (int c = 0; c < channels; ++c)
            memcpy(destination + frame + (size_t)c * 2,
                   source + frame + (size_t)mapping[c] * 2, 2);
}
#endif
