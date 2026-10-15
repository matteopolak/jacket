#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <initializer_list>
#define MEOW_DUMP 1
#include "../../../third_party/meow_hash/meow_hash_x64_aesni.h"

alignas(4096) static unsigned char input[8192];
static meow_dump trace[16];
static unsigned char expanded[128];

static bool write_bytes(const void *data, size_t count) {
    return fwrite(data, 1, count, stdout) == count;
}

static bool write_integer(uint64_t value, size_t count) {
    unsigned char bytes[8];
    for (size_t i = 0; i < count; ++i) bytes[i] = (unsigned char)(value >> (i * 8));
    return write_bytes(bytes, count);
}

static bool emit_case(uint64_t length, unsigned offset, unsigned mode) {
    unsigned char seed[128];
    memcpy(seed, mode == 4 ? expanded : MeowDefaultSeed, 128);
    if (mode == 1) seed[0] ^= 0x5a;
    if (mode == 2) MeowMaskLen[13] = 0;
    if (mode == 3) MeowShiftAdjust[0] = 1;
    unsigned char *source = input + 32 + offset;
    MeowDumpTo = trace;
    meow_u128 one = MeowHash(seed, length, source);
    meow_dump *one_seed = nullptr;
    for (meow_dump *p = trace; p < MeowDumpTo; ++p) {
        if (strcmp(p->Title, "PostMix") == 0) one_seed = p;
    }
    if (!one_seed || MeowDumpTo > trace + 16) return false;
    MeowDumpTo = nullptr;
    meow_state state;
    MeowBegin(&state, seed);
    uint64_t at = 0;
    while (at < length) {
        uint64_t chunk = (at % 7) * 17 + 1;
        if (chunk > length - at) chunk = length - at;
        MeowAbsorb(&state, chunk, source + at);
        at += chunk;
    }
    unsigned char streamed_seed[128];
    meow_u128 streamed = MeowEnd(&state, streamed_seed);
    bool ok = write_integer(length, 8) && write_integer(offset, 4) && write_integer(mode, 4)
        && write_bytes(&one, 16) && write_bytes(&streamed, 16)
        && write_bytes(one_seed->xmm, 128) && write_bytes(streamed_seed, 128);
    MeowMaskLen[13] = 255;
    MeowShiftAdjust[0] = 0;
    return ok;
}

int main(void) {
    for (size_t i = 0; i < sizeof(input); ++i) input[i] = (unsigned char)(i * 73 + 19);
    unsigned char abc[] = {'a', 'b', 'c'};
    MeowDumpTo = nullptr;
    MeowExpandSeed(3, abc, expanded);
    const unsigned extra[] = {47,48,49,63,64,65,95,96,97,127,128,129,159,160,161,
        191,192,193,223,224,225,239,240,241,254,255,256,257,258,511,512,513,1024,4095,4096,4097};
    const unsigned special[] = {0,3,32,33,255,256,257,4097};
    const unsigned expanded_lengths[] = {0,1,16,31,32,256,257,1024};
    const unsigned offsets[] = {0,1,7,15};
    const unsigned count = (34 + sizeof(extra) / sizeof(extra[0])) * 4 + 16 + 4 + 1 + 16;
    if (!write_bytes("MCV5", 4) || !write_integer(count, 4) || !write_bytes(expanded, 128)) return 2;
    for (unsigned length = 0; length <= 33; ++length)
        for (unsigned offset : offsets) if (!emit_case(length, offset, 0)) return 3;
    for (unsigned length : extra)
        for (unsigned offset : offsets) if (!emit_case(length, offset, 0)) return 3;
    for (unsigned length : special)
        for (unsigned offset : {0u, 7u}) if (!emit_case(length, offset, 1)) return 3;
    for (unsigned length : {3u, 259u})
        for (unsigned offset : {0u, 7u}) if (!emit_case(length, offset, 2)) return 3;
    if (!emit_case(3, 0, 3)) return 3;
    for (unsigned length : expanded_lengths)
        for (unsigned offset : {0u, 1u}) if (!emit_case(length, offset, 4)) return 3;
    return fflush(stdout) == 0 ? 0 : 2;
}
