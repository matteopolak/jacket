#import "Basic";

shifts :: (x: s32, small: u8, wide: u64, n: int) -> int {
    a := x << 31;

    b := (cast(s64) x) << 40;

    c := small << 7;

    d := wide >> 63;

    e := x << n;

    f: u64 = 1 << 40;

    g := x <<< 40;

    return a + cast(int) b + c + cast(int) d + e + cast(int) f + g;
}

main :: () {
    print("%\n", shifts(5, 1, 2, 3));
}
