// Which integer constants fit a type without a cast, and the type of an operation whose
// literal does not fit the other operand.
// rules: num.20 num.21 cast.35
#import "Basic";

ALL_BITS :: 0xff;

take_s8 :: (x: s8) -> s8 {
    return x;
}

main :: () {
    // Hex, binary and named constants may fill a signed type's bits.
    a: s8 = 0xff;
    b: s32 = 0xFFFF_FFFF;
    c: s8 = 0b1000_0000;
    d: s8 = ALL_BITS;
    assert(a == -1 && b == -1 && c == -128 && d == -1);
    assert(take_s8(0x80) == -128);
    // Every 64-bit literal fits s64.
    e: s64 = 9223372036854775808;
    assert(e == -9223372036854775807 - 1);
    f: u64 = ~0;
    assert(f == 0xFFFF_FFFF_FFFF_FFFF);
    assert(cast(s8) 0xff == -1 && cast(s32) 0xFFFF_FFFF == -1 && cast(s64) 0xFFFF_FFFF_FFFF_FFFF == -1);
    g := cast,trunc(u8) 300;
    h := cast,no_check(u8) -1;
    assert(g == 44 && h == 255);

    // A literal that does not fit the other operand makes the operation s64.
    small: u8 = 255;
    assert(type_of(small + -1) == s64 && small + -1 == 254);
    assert(!(small == -1));
    assert(type_of(small + 256) == s64 && small + 256 == 511);
    signed: s8 = 5;
    assert(type_of(signed + 200) == s64 && signed + 200 == 205);
    assert(type_of(signed + 0xff) == s8 && signed + 0xff == 4);
    // Typed constants meet in the wider type, like variables.
    assert(type_of(cast(s16) 1 + cast(s32) 2) == s32);
    assert(type_of(cast(u64) 1 + cast(u8) 2) == u64);
    // A bit-pattern literal keeps the operand's type.
    mask: u32 = 0xff;
    assert(type_of(mask & ~0x7) == u32 && mask & ~0x7 == 0xf8);
    print("ok\n");
}
