// popcount counts the set bits of each width, and agrees with a bit-by-bit count.
#import "Basic";
B :: #import "Bit_Operations";

// Counts the set bits of the low `width` bits of `x` one at a time.
count_bits :: (x: u64, width: int) -> s32 {
    n: s32 = 0;

    for i: 0..width - 1 {
        if ((x >> cast(u64) i) & 1) == 1 n += 1;
    }

    return n;
}

main :: () {
    assert(B.popcount(cast(u32) 0) == 0);
    assert(B.popcount(cast(u32) 0b1011) == 3);
    assert(B.popcount(cast(u8) 0xFF) == 8);
    assert(B.popcount(cast(s8) -1) == 8);
    assert(B.popcount(cast(s16) -1) == 16);
    assert(B.popcount(cast(s64) -1) == 64);
    assert(B.popcount(cast(u64) 0xFFFF_FFFF_FFFF_FFFF) == 64);

    mix: u64 = 0x9E37_79B9_7F4A_7C15;

    for i: 0..4095 {
        x := (cast(u64) i) * mix;
        assert(B.popcount(x) == count_bits(x, 64));
        assert(B.popcount(cast,trunc(u16) x) == count_bits(x, 16));
        assert(B.popcount(cast,trunc(u8) x) == count_bits(x, 8));
    }

    print("ok\n");
}
