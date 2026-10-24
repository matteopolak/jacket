#import "Basic";

Pair :: struct {
    a: s32;
    b: s32;
}

Swapped :: struct {
    b: s32;
    a: s32;
}

bits :: (a: s64, f: float32, p: Pair, n: u32) -> int {
    // Narrowing without the range check.
    x := cast,force(s32) a;

    // Reinterpreting the bits of a float.
    y := cast,force(u32) f;

    // Reinterpreting an aggregate.
    z := cast,force(Swapped) p;

    // Signed to unsigned: the plain cast checks the sign.
    w := cast,force(u64) a;

    // A plain cast already.
    v := cast(s64) n;

    return cast(int) x + cast(int) y + cast(int) z.a + cast(int) w + v;
}

main :: () {
    print("%\n", bits(1, 2, .{ 3, 4 }, 5));
}
