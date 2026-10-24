#import "Basic";

TIMEOUT :: 1000000;

main :: () {
    a := 123456;
    b := 1234567890;
    c: u32 = 0xdeadbeef;
    d: u64 = 0xFFFFFFFFFF;
    e := 0b101010101;
    f := 1234567.5;
    g := 100000000;

    print("% % % % % % %\n", TIMEOUT, a, b, c, d, e, f);
    print("%\n", g);
}
