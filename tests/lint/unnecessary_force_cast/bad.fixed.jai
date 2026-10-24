#import "Basic";

widen :: (a: s32, b: u8, c: float32, d: int) -> int {
    x := cast(s64) a;

    y := cast(u32) b;

    z := cast(float64) c;

    same := cast(int) d;

    return x + y + cast(int) z + same;
}

main :: () {
    print("%\n", widen(1, 2, 3, 4));
}
