ADPCM_CHANNELS_MAX :: 9;

Adpcm_Info :: struct {
    nChannels: s32;
    nBlockAlign: s32;
    wSamplesPerBlock: s32;
}

ADPCM_State :: struct {
    sample: s32;
    index: s8;
}

IMA_ADPCM_nibble :: inline (state: *ADPCM_State, nybble: u8) -> s32 {
    assert(nybble < 16);
    state.index = clamp(state.index, cast(s8) 0, cast(s8) 88);
    step := step_table[state.index];
    delta := step >> 3;

    if (nybble & 1) != 0 delta += step >> 2;
    if (nybble & 2) != 0 delta += step >> 1;
    if (nybble & 4) != 0 delta += step;
    if (nybble & 8) != 0 state.sample -= delta;
    else state.sample += delta;

    state.sample = clamp(state.sample, cast(s32) -32768, cast(s32) 32767);
    state.index += index_table[nybble];

    return state.sample;
}

Fill_IMA_ADPCM_block :: (dest: [] u8, dest_start_index: s32, encoded: [4] u8, channel: s32, numchannels: s32, state: *ADPCM_State) #no_alias {
    offset := dest_start_index + channel * 2;

    for byte: encoded {
        sample := IMA_ADPCM_nibble(state, byte & 15);

        store_sample(dest, offset, sample);
        offset += 2 * numchannels;
        sample = IMA_ADPCM_nibble(state, byte >> 4);
        store_sample(dest, offset, sample);
        offset += 2 * numchannels;
    }
}

IMA_ADPCM_decode :: (info: Adpcm_Info, input_block: [] u8, dest_buffer: [] u8) -> s32 {
    channels := info.nChannels;

    assert(channels > 0 && channels <= ADPCM_CHANNELS_MAX);
    assert(dest_buffer.count % (channels * 2) == 0);
    assert(info.nBlockAlign >= channels * 4 && info.wSamplesPerBlock > 0);
    source: s64;
    written: s32;
    block_bytes := info.wSamplesPerBlock * channels * 2;

    while source + info.nBlockAlign <= input_block.count && written + block_bytes <= dest_buffer.count {
        states: [ADPCM_CHANNELS_MAX] ADPCM_State;

        for c: 0..channels - 1 {
            offset := source + c * 4;
            sample := cast(u32) input_block[offset];
            sample |= (cast(u32) input_block[offset + 1]) << 8;
            states[c].sample = cast(s32) cast(s16) cast,truncate(u16) sample;
            states[c].index = cast,truncate(s8) input_block[offset + 2];
            store_sample(dest_buffer, written + cast(s32) c * 2, states[c].sample);
        }

        cursor := source + channels * 4;
        frame: s32 = 1;

        while frame + 7 < info.wSamplesPerBlock && cursor + channels * 4 <= source + info.nBlockAlign {
            for c: 0..channels - 1 {
                encoded: [4] u8;
                memcpy(encoded.data, input_block.data + cursor, 4);
                cursor += 4;
                Fill_IMA_ADPCM_block(dest_buffer, written + frame * channels * 2, encoded, cast(s32) c, channels, *states[c]);
            }

            frame += 8;
        }

        assert(frame == info.wSamplesPerBlock, "unsupported inconsistent ADPCM block");
        written += block_bytes;
        source += info.nBlockAlign;
    }

    return written;
}

step_table: [89] s32 : .[
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31,
    34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130,
    143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408,
    449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282,
    1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327,
    3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630,
    9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350,
    22385, 24623, 27086, 29794, 32767
];

index_table: [16] s8 : .[
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8
];

#scope_file
#import "Basic";

store_sample :: (dest: [] u8, offset: s32, sample: s32) {
    dest[offset] = cast,truncate(u8) sample;
    dest[offset + 1] = cast,truncate(u8) (sample >> 8);
}
