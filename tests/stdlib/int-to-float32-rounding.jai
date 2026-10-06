// A 64-bit integer converts to float32 with one rounding. Going through float64 first rounds
// twice: 2^63 + 2^39 + 1 becomes the f32 tie 2^63 + 2^39 in float64 and then rounds to even
// (down), although the integer is above the tie (found by tools/jaic-diff.py: the interpreter
// and the native build disagreed).
#import "Basic";

U :: 0x8000_0080_0000_0001; // 2^63 + 2^39 + 1
S :: 0x4000_0040_0000_0001; // 2^62 + 2^38 + 1
U_BITS :: 0x5f00_0001; // 2^63 + 2^40, the f32 above
S_BITS :: 0x5e80_0001; // 2^62 + 2^39

to_f32_u :: (x: u64) -> float32 {
    return cast(float32) x;
}
to_f32_s :: (x: s64) -> float32 {
    return cast(float32) x;
}

AT_COMPILE_TIME :: #run cast,force(u32) to_f32_u(U);

main :: () {
    u: u64 = U;
    s: s64 = S;
    assert(cast,force(u32) cast(float32) u == U_BITS);
    assert(cast,force(u32) cast(float32) s == S_BITS);
    assert(cast,force(u32) to_f32_u(u) == U_BITS);
    assert(cast,force(u32) to_f32_s(s) == S_BITS);
    assert(AT_COMPILE_TIME == U_BITS);
    // Folded constants.
    f: float32 = cast(float32) cast(u64) U;
    assert(cast,force(u32) f == U_BITS);
    g: float32 = cast(float32) cast(s64) S;
    assert(cast,force(u32) g == S_BITS);
    // Values float64 holds exactly are unaffected.
    small: s64 = -16777217;
    assert(cast(float32) small == -16777216.0);
    print("ok\n");
}
