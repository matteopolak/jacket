#import "Basic";

TIMEOUT :: 1_000_000;

main :: () {
    a := 123_456;
    b := 1_234_567_890;
    c: u32 = 0xdead_beef;
    d: u64 = 0xFF_FFFF_FFFF;
    e := 0b1_0101_0101;
    f := 1_234_567.5;
    g := 100_000_000;

    print("% % % % % % %\n", TIMEOUT, a, b, c, d, e, f);
    print("%\n", g);
}
