// The portable hash depends on the bytes, their length and the seed, and not on where in memory they sit.
#import "Basic";
#import "Extensions/Meow_Portable";
Meow :: #import "meow_hash";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

same_digest :: (a: [2] u64, b: [2] u64) -> bool {
    return a[0] == b[0] && a[1] == b[1];
}

main :: () {
    // Determinism, and the digest of "abc" from the module documentation.
    abc := cast([] u8) "abc";
    first := hash_bytes(abc);

    expect(same_digest(first, hash_bytes(abc)), "same input gives the same digest");
    expect(first[0] == 0x19f773ec430acc3b && first[1] == 0x5f6d236b424c9ab4, "digest of abc");

    // Length: the empty input, one zero byte and two zero bytes all differ.
    zeros: [600] u8;

    expect(!same_digest(hash_bytes(.[]), hash_bytes(([] u8).{ 1, zeros.data })), "empty and one zero byte differ");
    expect(!same_digest(hash_bytes(([] u8).{ 256, zeros.data }), hash_bytes(([] u8).{ 257, zeros.data })), "256 and 257 zero bytes differ");
    expect(!same_digest(hash_bytes(([] u8).{ 512, zeros.data }), hash_bytes(([] u8).{ 513, zeros.data })), "512 and 513 zero bytes differ");

    // Alignment: the same 300 bytes at addresses with different offsets mod 16 and 32.
    source: [400] u8;
    for 0..399 source[it] = cast,trunc(u8) (it * 7 + 3);

    aligned := hash_bytes(([] u8).{ 300, source.data });

    for offset: 1..40 {
        moved: [400] u8;
        memcpy(moved.data + offset, source.data, 300);
        expect(same_digest(hash_bytes(([] u8).{ 300, moved.data + offset }), aligned), tprint("300 bytes at offset %", offset));
    }

    // Changing one byte changes the digest, wherever it is.
    for position: 0..299 {
        changed := source;
        changed[position] ^= 1;
        expect(!same_digest(hash_bytes(([] u8).{ 300, changed.data }), aligned), tprint("flipping byte % changes the digest", position));
    }

    // Seed: the default seed and an all-zero seed give different digests for the same bytes, and a
    // seed with one changed byte gives a third.
    zero_seed: [128] u8;
    seeded := hash_bytes(abc, zero_seed);

    expect(!same_digest(seeded, first), "all-zero seed differs from the default seed");

    one_changed := Meow.MeowDefaultSeed;
    one_changed[100] ^= 0x80;
    expect(!same_digest(hash_bytes(abc, one_changed), first), "one changed seed byte changes the digest");
    expect(same_digest(hash_bytes(abc, Meow.MeowDefaultSeed), first), "passing the default seed explicitly matches");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
