// Test-time 16-bit mono sine sweep, deterministic header and no asset checked into git.
write_sweep :: (path: string, rate: s32 = 8000, frames: s64 = 8000) -> bool {
    bytes := NewArray(44 + frames * 2, u8);

    defer array_free(bytes);
    memcpy(bytes.data, "RIFF".data, 4);
    put32(bytes.data + 4, cast(u32) (bytes.count - 8));
    memcpy(bytes.data + 8, "WAVEfmt ".data, 8);
    put32(bytes.data + 16, 16);
    bytes[20] = 1;
    bytes[22] = 1;
    put32(bytes.data + 24, cast(u32) rate);
    put32(bytes.data + 28, cast(u32) rate * 2);
    bytes[32] = 2;
    bytes[34] = 16;
    memcpy(bytes.data + 36, "data".data, 4);
    put32(bytes.data + 40, cast(u32) frames * 2);
    phase: float64;

    for i: 0..frames - 1 {
        frequency := 110.0 + 990.0 * cast(float64) i / frames;
        value := cast(s16) (sin(phase) * 12000);

        phase += TAU64 * frequency / rate;
        bits := cast,no_check(u16) value;

        bytes[44 + i * 2] = cast,trunc(u8) bits;
        bytes[45 + i * 2] = cast,trunc(u8) (bits >> 8);
    }

    return write_entire_file(path, .{ bytes.count, bytes.data });
}

#scope_file
put32 :: (p: *u8, value: u32) {
    for i: 0..3 p[i] = cast,trunc(u8) (value >> (i * 8));
}
