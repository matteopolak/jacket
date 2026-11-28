the_accumulator: [48000] Audio_Dest_Sample;
accumulator_element_size :: size_of(Audio_Dest_Sample);

// Returns the channel-wise sum of `v0` and `v1` over the first `nchannels` channels; the rest are 0.
add :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base, nchannels: s64) -> Audio_Sample_Base {
    B.assert(v0 != null && v1 != null && nchannels >= 0 && nchannels <= MAX_CHANNELS_PER_SOUND, "audio sample input/channel range invalid");
    result: Audio_Sample_Base;
    for 0..nchannels - 1 result.channels[it] = v0.channels[it] + v1.channels[it];
    return result;
}

// Adds the first `nchannels` channels of `v1` to `v0`.
plus_equals :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base, nchannels: s64) {
    B.assert(v0 != null && v1 != null && nchannels >= 0 && nchannels <= MAX_CHANNELS_PER_SOUND, "audio sample input/channel range invalid");
    for 0..nchannels - 1 v0.channels[it] += v1.channels[it];
}

// Returns the channel-wise product of `v0` and `v1`.
multiply :: (v0: *Audio_Sample_Base, v1: *Audio_Sample_Base) -> Audio_Sample_Base {
    B.assert(v0 != null && v1 != null);
    result: Audio_Sample_Base;
    for 0..MAX_CHANNELS_PER_SOUND - 1 result.channels[it] = v0.channels[it] * v1.channels[it];
    return result;
}

// Returns `v0` with every channel multiplied by `k`.
multiply :: (v0: *Audio_Sample_Base, k: float) -> Audio_Sample_Base {
    B.assert(v0 != null);
    result: Audio_Sample_Base;
    for 0..MAX_CHANNELS_PER_SOUND - 1 result.channels[it] = v0.channels[it] * k;
    return result;
}

// Returns `v0` with every channel multiplied by `k`.
multiply :: (k: float, v0: *Audio_Sample_Base) -> Audio_Sample_Base {
    return multiply(v0, k);
}

// Multiplies every channel of `v0` by `k`.
times_equals :: (v0: *Audio_Sample_Base, k: float) {
    B.assert(v0 != null);
    for 0..MAX_CHANNELS_PER_SOUND - 1 v0.channels[it] *= k;
}

// Reads four bytes of `data` from `base` as a little-endian 32-bit number, as used for file tags.
// Stops the program if fewer than four bytes remain.
get_magic4 :: (data: string, base: s64) -> u32 {
    B.assert(base >= 0 && data.count >= 4 && base <= data.count - 4, "audio magic read outside input");
    a := cast(u32) data[base];
    b := cast(u32) data[base + 1];
    c := cast(u32) data[base + 2];
    d := cast(u32) data[base + 3];

    return a | (b << 8) | (c << 16) | (d << 24);
}

// Splits a frame position into the frame index below it and the fraction past that frame, 0 up to 1.
//
// Stops the program for a position that is not finite or does not fit an index.
get_icursor_and_remainder :: (source_cursor: float64) -> icursor: s64, remainder: float64 {
    assert(is_finite(source_cursor));
    integral := floor(source_cursor);

    assert(integral >= -9223372036854775808.0 && integral < 9223372036854775808.0);
    index := cast(s64) integral;

    return index, source_cursor - integral;
}

#scope_file
B :: #import "Basic";
#import "Math";
#import "Basic";
