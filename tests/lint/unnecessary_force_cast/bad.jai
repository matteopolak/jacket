#import "Basic";

widen :: (a: s32, b: u8, c: float32, d: int) -> int {
    x := cast,force(s64) a;

    y := cast,force(u32) b;

    z := cast,force(float64) c;

    same := cast,force(int) d;

    return x + y + cast(int) z + same;
}

main :: () {
    print("%\n", widen(1, 2, 3, 4));
}
