#import "Basic";
#import "Extensions/Meow_Portable";
#import "meow_hash";
F :: #import "File";
S :: #import "String";

main :: () {
    abc := hash_bytes(cast([] u8) "abc");

    assert(abc[0] == 0x19f773ec430acc3b && abc[1] == 0x5f6d236b424c9ab4);
    empty := hash_bytes(.[]);

    assert(empty[0] == 0x657ca02a5859c045 && empty[1] == 0x75a7b5550383265e);
    // Compare the register fast path too when running on x64.
    bytes: [1000] u8;
    for *bytes it.* = cast(u8) ((it_index * 37 + 11) & 255);

    #if CPU == .X64 {
        for count: int.[1, 15, 16, 17, 31, 32, 33, 100, 255, 256, 257, 300, 511, 512, 700, 1000] {
            portable := hash_bytes(([] u8).{ count, bytes.data });
            #asm { pxor hash:, hash; }
            MeowHash(MeowDefaultSeed, cast(u64) count, bytes.data, hash);
            assert(portable[0] == MeowU64From(hash, 0) && portable[1] == MeowU64From(hash, 1));
        }
    }

    check_fixture();
    print("ok\n");
}

read_le :: (p: *u8, count: int) -> u64 {
    result: u64;
    for 0..count - 1 result |= (cast(u64) p[it]) << (it * 8);
    return result;
}

check_fixture :: () {
    path := S.join(S.path_strip_filename(#file), "../../../meow_hash/tests/fixtures/vectors.bin");

    defer free(path);
    fixture, ok := F.read_entire_file(path);

    assert(ok && fixture.count == 96504);
    defer free(fixture);
    bytes := cast(*u8) fixture.data;
    input: [8192] u8;

    for *input it.* = cast(u8) ((it_index * 73 + 19) & 255);

    checked := 0;

    for record: 0..316 {
        p := bytes + 136 + record * 304;
        length := cast(int) read_le(p, 8);
        offset := cast(int) read_le(p + 8, 4);
        mode := read_le(p + 12, 4);

        // Modes 2/3 deliberately mutate the register API's shuffle/mask globals.
        if mode == 2 || mode == 3 continue;

        seed := MeowDefaultSeed;
        if mode == 1 seed[0] ^= 0x5A;
        if mode == 4 for 0..127 seed[it] = bytes[8 + it];

        digest := hash_bytes(([] u8).{ length, input.data + 32 + offset }, seed);

        assert(digest[0] == read_le(p + 16, 8) && digest[1] == read_le(p + 24, 8));
        checked += 1;
    }

    assert(checked == 312);
}
