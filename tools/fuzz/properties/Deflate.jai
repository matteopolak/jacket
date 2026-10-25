// Levels -1 through 10: every level plus one out of range at each end, which are clamped.
LEVEL_CHOICES :: 12;

// Compressing then decompressing returns the input, for every container and level, and
// inflate_into agrees with inflate.
fuzz_deflate_round_trip :: (r: *Rng) {
    input := gen(r, string);
    data: [] u8;
    data.data = input.data;
    data.count = input.count;

    container := cast(Target.Deflate_Container) (gen(r, u8) % 3);
    level := cast(s32) (gen(r, u8) % LEVEL_CHOICES) - 1;

    packed := Target.deflate(data, container, level);
    output, ok, error := Target.inflate(packed, container);

    assert(ok, "deflate output inflates");
    assert(output.count == data.count, "round trip length");

    for 0..data.count - 1 assert(output[it] == data[it], "round trip bytes");

    exact: [] u8;
    exact.data = alloc(max(data.count, 1));
    exact.count = data.count;

    written, ok2 := Target.inflate_into(packed, exact, container);

    assert(ok2 && written == data.count, "inflate_into fits exactly");

    for 0..data.count - 1 assert(exact[it] == data[it], "inflate_into bytes");

    free(exact.data);
    free(output.data);
    free(packed.data);
}

// Arbitrary bytes are decoded or rejected, never worse, and anything accepted can be packed again.
fuzz_inflate_random_bytes :: (r: *Rng) {
    input := gen(r, string);
    data: [] u8;
    data.data = input.data;
    data.count = input.count;

    for container: Target.Deflate_Container.[.RAW, .ZLIB, .GZIP] {
        output, ok, error := Target.inflate(data, container, 1 << 24);

        if ok {
            again := Target.deflate(output, .RAW, 1);
            back, ok2 := Target.inflate(again, .RAW);

            assert(ok2 && back.count == output.count, "accepted output survives a round trip");
            free(back.data);
            free(again.data);
            free(output.data);
        } else {
            assert(error.count > 0, "failure carries a reason");
        }
    }
}

// A valid stream with a few bytes changed or cut: any answer, no crash.
fuzz_inflate_damaged :: (r: *Rng) {
    input := gen(r, string);
    data: [] u8;
    data.data = input.data;
    data.count = input.count;

    container := cast(Target.Deflate_Container) (gen(r, u8) % 3);
    packed := Target.deflate(data, container, cast(s32) (gen(r, u8) % 10));

    if packed.count > 0 {
        edits := 1 + gen(r, u8) % 4;

        for 0..cast(s64) edits - 1 {
            at := cast(s64) (gen(r, u32) % cast(u32) packed.count);
            packed[at] ^= cast(u8) (1 + gen(r, u8) % 255);
        }

        cut := cast(s64) (gen(r, u32) % cast(u32) (packed.count + 1));

        packed.count = cut;
    }

    output, ok := Target.inflate(packed, container, 1 << 24);
    if ok free(output.data);

    free(packed.data);
}

// Checksums of the two halves, continued, equal the checksum of the whole.
fuzz_checksum_chaining :: (r: *Rng) {
    input := gen(r, string);
    data: [] u8;
    data.data = input.data;
    data.count = input.count;

    split := cast(s64) 0;
    if data.count > 0 split = cast(s64) (gen(r, u64) % cast(u64) (data.count + 1));

    head: [] u8;
    head.data = data.data;
    head.count = split;

    tail: [] u8;
    tail.data = data.data + split;
    tail.count = data.count - split;

    assert(Target.crc32(tail, Target.crc32(head)) == Target.crc32(data), "crc32 chaining");
    assert(Target.adler32(tail, Target.adler32(head)) == Target.adler32(data), "adler32 chaining");
}
