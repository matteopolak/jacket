#import "Basic";

TIMEOUT :: 1_000_000;

// A macro may paste a literal as it was given.
big :: () -> int #expand {
    return 12345678;
}

main :: () {
    a := 12345;
    b := 65536;
    c: u32 = 0xdead_beef;
    d := 0xFFFF;
    e := 0b1010;
    f := 1_234_567.5;
    g := 12345.678;
    h := 1e9;
    i := 0xFFFFFFF;
    j := 1000_000;

    print("% % % % % % % % % %\n", TIMEOUT, a, b, c, d, e, f, g, h, i);
    print("% %\n", j, big());
}
