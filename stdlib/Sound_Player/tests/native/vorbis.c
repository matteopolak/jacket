/* Pinned public stb header only; decoder implementation stays opaque.
   Real decoder consumer, never an audio-device consumer. */
#include <stdio.h>
#include <stdlib.h>
#include <limits.h>
#include <string.h>
#define STB_VORBIS_HEADER_ONLY
#include "stb_vorbis.c"

#define CHECK(condition) do { if (!(condition)) { failed_line = __LINE__; goto done; } } while (0)
int main(int argc, char **argv) {
    FILE *input = NULL;
    unsigned char *bytes = NULL;
    stb_vorbis *decoder = NULL;
    int failed_line = 0, error = 0;
    CHECK(argc == 2);
#if defined(_WIN32)
    CHECK(!fopen_s(&input, argv[1], "rb"));
#else
    input = fopen(argv[1], "rb");
#endif
    CHECK(input);
    CHECK(!fseek(input, 0, SEEK_END));
    long size = ftell(input);
    CHECK(size > 0 && size <= INT_MAX);
    CHECK(!fseek(input, 0, SEEK_SET));
    bytes = malloc((size_t)size);
    CHECK(bytes);
    CHECK(fread(bytes, 1, (size_t)size, input) == (size_t)size);
    CHECK(!ferror(input));
    int closed = fclose(input);
    input = NULL;
    CHECK(!closed);
    decoder = stb_vorbis_open_memory(bytes, (int)size, &error, NULL);
    CHECK(decoder && !error);
    stb_vorbis_info info = stb_vorbis_get_info(decoder);
    CHECK(info.channels == 2 && info.sample_rate == 44100);
    CHECK(stb_vorbis_stream_length_in_samples(decoder) == 4416);
    short pcm[4416 * 2], sought[64 * 2];
    int frames = stb_vorbis_get_samples_short_interleaved(decoder, 2, pcm, 4416 * 2);
    CHECK(frames == 4416);
    double energy[2] = {0, 0};
    for (int frame = 0; frame < frames; ++frame)
        for (int channel = 0; channel < 2; ++channel) {
            double value = pcm[frame * 2 + channel] / 32768.0;
            energy[channel] += value * value;
        }
    CHECK(energy[0] / frames > 0.025 && energy[0] / frames < 0.04);
    CHECK(energy[1] / frames > 0.10 && energy[1] / frames < 0.15);
    CHECK(stb_vorbis_seek(decoder, 1000));
    CHECK(stb_vorbis_get_samples_short_interleaved(decoder, 2, sought, 64 * 2) == 64);
    CHECK(!memcmp(sought, pcm + 1000 * 2, sizeof(sought)));
done:
    if (decoder) stb_vorbis_close(decoder);
    free(bytes);
    if (input && fclose(input) && !failed_line) failed_line = __LINE__;
    if (failed_line) {
        fprintf(stderr, "Vorbis consumer check failed at line %d\n", failed_line);
        return 1;
    }
    puts("ok");
    return 0;
}
