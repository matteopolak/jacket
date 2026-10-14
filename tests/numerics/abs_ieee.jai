#import "Basic";
M :: #import "Math";

B32 :: union {
    f: float32;
    u: u32;
}

B64 :: union {
    f: float64;
    u: u64;
}

main :: () {
    x: B32;
    y: B32;

    x.u = 0x80000000;
    y.f = M.abs(x.f);
    assert(y.u == 0, "abs must clear the sign of negative zero");
    x.u = 0xffc12345;
    y.f = M.abs(x.f);
    assert(y.u == 0x7fc12345);
    a: B64;
    b: B64;

    a.u = 0x8000000000000000;
    b.f = M.abs(a.f);
    assert(b.u == 0);
    a.u = 0xfff8123456789abc;
    b.f = M.abs(a.f);
    assert(b.u == 0x7ff8123456789abc);
    assert(M.abs(-M.FLOAT64_INFINITY) == M.FLOAT64_INFINITY);
    print("IEEE absolute value ok\n");
}
