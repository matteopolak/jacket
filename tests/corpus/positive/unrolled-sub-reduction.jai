// Reduced from a tools/jaigen.py program (seed 10014) on which native -O2 disagreed with the
// interpreter: LLVM 22's runtime unroller split the 'v29.a -= v29.b' recurrence across parallel
// accumulators and recombined them wrongly. jaic disables that transformation.
#import "Basic";
H: u64 = 0xcbf29ce484222325;
mix :: (v: u64) { H = (H ^ v) * 0x100000001b3; }
wrap_index :: (v: s64, n: s64) -> s64 { r := v % n; if r < 0 r += n; return r; }
Pair :: struct (T: Type) { a: T; b: T; }
f_to_u8_float32 :: (v: float32, lo: float64, hi: float64) -> u8 { if cast(float64) v >= lo && cast(float64) v <= hi return cast(u8) v; return 8; }
safe_rem_u64 :: (a: u64, b: u64) -> u64 { if b == 0 return a; return a % b; }
E1 :: enum u16 {
    E1_C;
}
S2 :: struct {
    f0: u8;
}
S3 :: struct {
    f0: s64;
    f1: u64 = cast(u64) 0xe;
    f2: s8 = cast(s8) 1;
    f3: u64;
    arr: [3] u32;
    inner: S2;
}
poly_4 :: (a: $T, b: T) -> T {
    return (a | cast(T) 3);
}
g5: s64 = cast(s64) (-13);
act_16 :: (a0: S2, a1: Pair(s64), a2: u8, a3: S3) -> bool {
    v19: float64 = cast(float64) 2147483647.5;
    print("hex %\n", formatInt(g5, base = 2));
    return ((-v19) == v19);
}
main :: () {
    v29: Pair(s64) = Pair(s64).{cast(s64) 4097, (g5 + cast(s64) (-18))};
    v30: Pair(u64) = Pair(u64).{safe_rem_u64(cast(u64) 4, cast(u64) 20), (cast,no_check(u64) E1.E1_C)};
    v32: [..] u8;
    array_add(*v32, ((cast,no_check(u8) v30.a) & cast(u8) 0x7));
    array_add(*v32, f_to_u8_float32(cast(float32) 1e10, 0, 255));
    array_add(*v32, poly_4(cast(u8) 186, cast(u8) 8));
    v35: s16 = ((cast(s16) 32766 >>> (cast(s16) (-11) & 15)) % ((cast(s16) 0 & 7) | 1));
    array_add(*v32, (0 - cast(u8) 0x1));
    for v32 {
        mix(cast,no_check(u64) it);
        v29.a -= v29.b;
    }
    v37: [3] S3 = S3.[S3.{f0 = v29.b, f1 = v30.b, arr = u32.[cast(u32) 0x8, cast(u32) 0x11, cast(u32) 0]}, S3.{f0 = g5, f1 = v30.a, f2 = cast(s8) (-127), f3 = v30.a, arr = u32.[cast(u32) 0x0, cast(u32) 0xda1193b3, cast(u32) 16], inner = S2.{f0 = v32[wrap_index(cast,no_check(s64) (cast(u8) 12), v32.count)]}}, S3.{f0 = cast(s64) (-8328108846637510514), f1 = v30.a, f3 = (v30.a % ((v30.b & 127) | 1)), arr = u32.[cast(u32) 0, cast(u32) 2783243123, cast(u32) 3153832184]}];
    v52: bool = act_16(v37[wrap_index(cast,no_check(s64) (v35), 3)].inner, v29, cast(u8) 0x7a, v37[wrap_index(cast,no_check(s64) (v29.b), 3)]);
    mix(cast,no_check(u64) v29.a);
    print("checksum %\n", formatInt(H, base = 16));
}
