#import "Basic";

shifts :: (x: s32, small: u8, wide: u64) -> int {
    a := x << 40;

    b := x >> 32;

    c := small << 8;

    d := wide >> 64;

    e := x;
    e <<= 33;

    return a + b + c + cast(int) d + e;
}

main :: () {
    print("%\n", shifts(5, 1, 2));
}
