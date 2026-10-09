// lz4 against the pinned C library (tools/build_native_libs.py): compress and decompress buffers of
// different shapes (repetitive, text-like, incompressible), the bound on the compressed size, the
// fast and high-compression modes, a destination that is too small, truncated and corrupt input,
// and decompressing only a prefix.
#import "Basic";
#import "lz4";

compress :: (input: [] u8, level: s32) -> [] u8 {
    capacity := LZ4_compressBound(cast(s32) input.count);
    assert(capacity >= input.count, "the bound covers the input");
    out := NewArray(capacity, u8);
    size: s32;
    if level == 0 size = LZ4_compress_default(input.data, out.data, cast(s32) input.count, capacity);
    else size = LZ4_compress_HC(input.data, out.data, cast(s32) input.count, capacity, level);
    assert(size > 0, "compression produced % bytes", size);
    out.count = size;
    return out;
}

round_trip :: (name: string, input: [] u8, expect_smaller: bool) {
    for level: s32.[0, 1, 9, 12] {
        packed := compress(input, level);
        if expect_smaller assert(packed.count < input.count / 2, "% at level %: % -> % bytes", name, level, input.count, packed.count);
        back := NewArray(input.count, u8);
        size := LZ4_decompress_safe(packed.data, back.data, cast(s32) packed.count, cast(s32) back.count);
        assert(size == input.count, "% at level %: decompressed % of % bytes", name, level, size, input.count);
        for i: 0..input.count - 1 assert(back[i] == input[i], "% at level %: byte % differs", name, level, i);
        array_free(packed);
        array_free(back);
    }
}

main :: () {
    assert(LZ4_versionNumber() >= 10904, "version %", LZ4_versionNumber());
    version := to_string(LZ4_versionString());
    assert(version.count > 0 && version[0] == #char "1", "version string %", version);

    // Repetitive data shrinks a lot.
    zeros := NewArray(100_000, u8);
    round_trip("zeros", zeros, true);

    text := NewArray(60_000, u8);
    phrase := "the quick brown fox jumps over the lazy dog; ";
    for 0..text.count - 1 text[it] = phrase[it % phrase.count];
    round_trip("text", text, true);

    // Pseudo-random bytes do not compress; the output still stays within the bound and round-trips.
    noise := NewArray(50_000, u8);
    state: u32 = 12345;
    for 0..noise.count - 1 {
        state = state * 1664525 + 1013904223;
        noise[it] = cast,trunc(u8) (state >> 24);
    }
    round_trip("noise", noise, false);

    // Tiny inputs, including one byte.
    one := u8.[42];
    round_trip("one byte", one, false);
    short := u8.[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    round_trip("short", short, false);

    // The bound grows with the input and exceeds it.
    assert(LZ4_compressBound(0) > 0 && LZ4_compressBound(1000) > 1000 && LZ4_compressBound(2000) > LZ4_compressBound(1000));

    // High compression is not worse than the default mode on text; fast mode with a large
    // acceleration is not better than it.
    default_size := compress(text, 0).count;
    hc_size := compress(text, 9).count;
    assert(hc_size <= default_size, "HC % vs default %", hc_size, default_size);
    fast := NewArray(LZ4_compressBound(cast(s32) text.count), u8);
    fast_size := LZ4_compress_fast(text.data, fast.data, cast(s32) text.count, cast(s32) fast.count, 50);
    assert(fast_size > 0 && fast_size >= default_size, "fast % vs default %", fast_size, default_size);

    // A destination that is too small fails compression and decompression instead of overflowing.
    small := NewArray(16, u8);
    assert(LZ4_compress_default(text.data, small.data, cast(s32) text.count, cast(s32) small.count) == 0, "no room to compress");
    packed := compress(text, 0);
    short_out := NewArray(text.count - 1, u8);
    assert(LZ4_decompress_safe(packed.data, short_out.data, cast(s32) packed.count, cast(s32) short_out.count) < 0, "no room to decompress");

    // Truncated and corrupt input are rejected.
    back := NewArray(text.count, u8);
    assert(LZ4_decompress_safe(packed.data, back.data, cast(s32) packed.count / 2, cast(s32) back.count) < 0, "truncated input");
    garbage := u8.[0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x01];
    assert(LZ4_decompress_safe(garbage.data, back.data, garbage.count, cast(s32) back.count) < 0, "garbage input");

    // Decompressing only the first part of the data.
    prefix := NewArray(1000, u8);
    got := LZ4_decompress_safe_partial(packed.data, prefix.data, cast(s32) packed.count, 1000, cast(s32) prefix.count);
    assert(got == 1000, "partial decompression gave % bytes", got);
    for i: 0..999 assert(prefix[i] == text[i], "prefix byte %", i);

    print("ok\n");
}
