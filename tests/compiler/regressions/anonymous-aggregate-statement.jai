// A bare `union { ... }` or `struct { ... }` statement brings its members into scope: in a
// struct body (as a member) and in a procedure body (as an anonymous local).
#import "Basic";

Value :: struct {
    kind: int;
    union {
        i: s32;
        f: float32;
    }
}

main :: () {
    v: Value;
    v.f = 1.0;
    assert(v.i == 0x3f800000);

    union {
        x: s32;
        y: float32;
    }

    x = 5;
    assert(x == 5);
    y = 1.0;
    assert(x == 0x3f800000);

    struct {
        lo: s32;
        hi: s32;
    }

    lo = 1;
    hi = 2;
    assert(lo + hi == 3);

    print("ok\n");
}
