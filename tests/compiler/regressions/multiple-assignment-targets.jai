// In `a, b = xx x, xx y` each `xx` converts to the type of its own target, and `_` in a mixed
// declaration list discards its value.
#import "Basic";

V :: struct {
    x, y: float64;
}

three :: () -> bool, s64, s64 {
    return true, 1, 2;
}

main :: () {
    v: V;
    s := s64.[1, 2];

    v.x, v.y = xx s[0], xx s[1];
    assert(v.x == 1 && v.y == 2, "% %", v.x, v.y);

    n: s32;
    f: float;
    n, f = xx 7.9, xx 3;
    assert(n == 7 && f == 3, "% %", n, f);

    ok: bool;
    ok, count:, _ = three();

    assert(ok && count == 1, "% %", ok, count);
    print("ok\n");
}
