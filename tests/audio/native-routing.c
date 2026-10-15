/* Controlled bytes only: these nonzero channel markers never reach a device. */
#include "routing.h"
#include <stdio.h>
#define CHECK(condition) do { if (!(condition)) return 1; } while (0)
int main(void) {
    uint8_t input[35] = {0}, output[35] = {0};
    // ALSA's common order: FL FR RL RR FC LFE SL SR.
    unsigned order[8] = {0, 1, 4, 5, 2, 3, 6, 7};
    CHECK(audio_valid_channel_order(order, 8));
    for (unsigned frame = 0; frame < 2; ++frame)
        for (unsigned c = 0; c < 8; ++c) {
            input[1 + frame * 16 + c * 2] = (uint8_t)(frame * 16 + c);
            input[2 + frame * 16 + c * 2] = (uint8_t)(0x80 + c);
        }
    audio_copy_channels(output + 1, input + 1, 32, 8, order);
    for (unsigned frame = 0; frame < 2; ++frame)
        for (unsigned c = 0; c < 8; ++c) {
            CHECK(output[1 + frame * 16 + c * 2] == frame * 16 + order[c]);
            CHECK(output[2 + frame * 16 + c * 2] == 0x80 + order[c]);
        }
    CHECK(output[0] == 0 && output[33] == 0 && output[34] == 0);
    order[0] = order[1];
    CHECK(!audio_valid_channel_order(order, 8));
    for (unsigned c = 0; c < 6; ++c) order[c] = c;
    CHECK(audio_valid_channel_order(order, 6));
    audio_copy_channels(output + 1, input + 1, 24, 6, order);
    CHECK(memcmp(output + 1, input + 1, 24) == 0);
    puts("ok");
    return 0;
}
