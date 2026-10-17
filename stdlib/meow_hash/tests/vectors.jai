#import "Basic";
M :: #import "meow_hash";
F :: #import "File";
S :: #import "String";

read_integer :: (p: *u8, count: s64) -> u64 {
    result: u64;

    for i: 0..count - 1 {
        byte := cast(u64) p[i];
        result |= byte << (i * 8);
    }

    return result;
}

main :: () {
    #assert !#exists(M.finish);
    #assert !#exists(M.absorb_state_blocks);
    filename := S.join(S.path_strip_filename(#file), "fixtures/vectors.bin");

    defer free(filename);
    bytes, loaded := F.read_entire_file(filename);

    assert(loaded && bytes.count == 96504);
    defer free(bytes);
    records := cast(*u8) bytes.data;

    assert(records[0] == 77 && records[1] == 67 && records[2] == 86 && records[3] == 53);
    assert(read_integer(records + 4, 4) == 317);
    expanded: [128] u8;

    memcpy(expanded.data, records + 8, 128);
    input, base := NewArray(8192, u8, alignment = 4096);

    defer free(base);
    for 0..8191 input[it] = cast,truncate(u8) (it * 73 + 19);

    saved_seed := M.MeowDefaultSeed[0];
    saved_mask := M.MeowMaskLen[13];
    saved_shift := M.MeowShiftAdjust[0];

    defer {
        M.MeowDefaultSeed[0] = saved_seed;
        M.MeowMaskLen[13] = saved_mask;
        M.MeowShiftAdjust[0] = saved_shift;
    }

    for record: 0..316 {
        p := records + 136 + record * 304;
        length := read_integer(p, 8);
        offset := read_integer(p + 8, 4);
        mode := read_integer(p + 12, 4);

        assert(length <= 4097 && offset <= 15 && mode <= 4);
        M.MeowDefaultSeed[0] = saved_seed;
        M.MeowMaskLen[13] = saved_mask;
        M.MeowShiftAdjust[0] = saved_shift;
        if mode == 1 M.MeowDefaultSeed[0] ^= 0x5a;
        if mode == 2 M.MeowMaskLen[13] = 0;
        if mode == 3 M.MeowShiftAdjust[0] = 1;

        seed := M.MeowDefaultSeed;
        if mode == 4 seed = expanded;

        source := input.data + 32 + offset;

        #asm { pxor one:, one; pxor streamed:, streamed; }
        M.MeowHash(seed, length, source, one);
        assert(M.MeowU64From(one, 0) == read_integer(p + 16, 8));
        assert(M.MeowU64From(one, 1) == read_integer(p + 24, 8));
        state: M.meow_state;

        M.MeowBegin(*state, seed);
        at: u64;
        while at < length {
            chunk := min((at % 7) * 17 + 1, length - at);
            M.MeowAbsorb(*state, chunk, source + at);
            at += chunk;
        }

        assert(state.TotalLengthInBytes == length && state.BufferLen == (length & 255));
        state_out: [128] u8;

        M.MeowEnd(*state, streamed, state_out.data);
        assert(M.MeowU64From(streamed, 0) == read_integer(p + 32, 8));
        assert(M.MeowU64From(streamed, 1) == read_integer(p + 40, 8));
        for 0..127 assert(state_out[it] == p[176 + it]);

        M.MeowEnd(*state, streamed);
        assert(M.MeowU64From(streamed, 0) == read_integer(p + 32, 8));
        assert(M.MeowU64From(streamed, 1) == read_integer(p + 40, 8));

        #asm {
            pxor h0:, h0; pxor h1:, h1; pxor h2:, h2; pxor h3:, h3;
            pxor h4:, h4; pxor h5:, h5; pxor h6:, h6; pxor h7:, h7;
            pxor r1:, r1; pxor r3:, r3;
        }

        M.MeowBegin(seed, h0, h1, h2, h3, h4, h5, h6, h7);
        blocks := length >> 8;

        M.MeowAbsorbBlocks(blocks, source, h0, h1, h2, h3, h4, h5, h6, h7);
        direct_out: [128] u8;

        M.MeowEnd(length, source + (blocks << 8), r1, r3, h0, h1, h2, h3, h4, h5, h6, h7, direct_out.data);
        assert(M.MeowU64From(h0, 0) == read_integer(p + 16, 8));
        assert(M.MeowU64From(h0, 1) == read_integer(p + 24, 8));
        for 0..127 assert(direct_out[it] == p[48 + it]);
    }

    print("ok\n");
}
