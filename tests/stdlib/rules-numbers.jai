// rules: num.1 num.2 num.3 num.4 num.5 num.8 num.9 num.10 num.11 num.12 num.13 num.14
// Self-checking program for the claims in docs/language/numbers.md.
#import "Basic";

which :: (x: float32) -> int {
    return 32;
}
which :: (x: float64) -> int {
    return 64;
}

main :: () {
    // num.1: int is s64.
    #assert int == s64;
    #assert size_of(int) == 8;

    // num.2: an untyped integer literal settles as s64.
    small := 5;
    #assert type_of(small) == s64;

    // num.3: an untyped float literal is float32 unless it needs float64 precision.
    short := 0.75;
    long := 9876543210.125;
    #assert type_of(short) == float32;
    #assert type_of(long) == float64;

    // num.4: an untyped float literal prefers a float32 overload.
    assert(which(1.5) == 32, "untyped float literal picks the float32 overload");

    // num.5: widening conversions are implicit.
    byte: u8 = 200;
    half: u16 = byte;
    word: u32 = half;
    signed_wide: s32 = byte;
    tiny: s8 = -5;
    wide_signed: s64 = tiny;
    f32: float32 = 1.25;
    f64: float64 = f32;
    assert(half == 200 && word == 200 && signed_wide == 200, "unsigned widening keeps the value");
    assert(wide_signed == -5, "signed widening keeps the sign");
    assert(f64 == 1.25, "float32 widens to float64");

    // num.8: division and remainder truncate toward zero.
    m7 := -7;
    two := 2;
    three := 3;
    assert(m7 / two == -3, "-7 / 2 is -3");
    assert(m7 % three == -1, "-7 mod 3 is -1");
    assert(7 / -two == -3, "7 / -2 is -3");
    assert(7 % -three == 1, "7 mod -3 is 1");
    #assert -7 / 2 == -3;
    #assert -7 % 3 == -1;

    // num.9: u64 accepts its maximum as a literal.
    top: u64 = 18446744073709551615;
    assert(top + 1 == 0, "u64 max wraps to 0");
    assert(top == 0xFFFF_FFFF_FFFF_FFFF, "u64 max literal");

    // num.10: the most negative s64 is a valid literal.
    bottom: s64 = -9223372036854775808;
    assert(bottom < 0 && bottom - 1 > 0, "s64 minimum literal");

    // num.11: digit separators, hex and binary literals.
    assert(1_000 == 1000, "underscore separator");
    assert(0xFF_FF == 65535, "hex with separator");
    assert(0b1010 == 10, "binary literal");

    // num.12: float division by zero gives infinities.
    zero := 0.0;
    pos := 1.0 / zero;
    neg := -1.0 / zero;
    assert(pos > 1e38 && pos == pos * 2, "1/0 is +inf");
    assert(neg < -1e38 && neg == neg * 2, "-1/0 is -inf");

    // num.13: integer arithmetic wraps at the declared width.
    v: u8 = 255;
    v += 1;
    assert(v == 0, "u8 255 + 1 wraps to 0");
    s: s8 = 127;
    s += 1;
    assert(s == -128, "s8 127 + 1 wraps to -128");
    w: u16 = 0;
    w -= 1;
    assert(w == 65535, "u16 0 - 1 wraps to 65535");

    // num.14: by default overflow is not checked (the program keeps running).
    big: s32 = 0x7FFF_FFFF;
    big *= 2;
    assert(big == -2, "s32 overflow wraps silently");

    print("ok\n");
}
