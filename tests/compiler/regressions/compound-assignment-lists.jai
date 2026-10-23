// `a, b += x, y` pairs targets with values, reading every value before any target changes.
#import "Basic";

V :: struct {
    x, y: float;
}

main :: () {
    a, b := 1, 2;
    a, b += 3, 4;
    assert(a == 4 && b == 6, "% %", a, b);

    a, b = 1, 2;
    a, b += b, a;
    assert(a == 3 && b == 3, "% %", a, b);

    v: V;
    v.x, v.y = 1, 2;
    v.x, v.y *= 5, 10;
    assert(v.x == 5 && v.y == 20, "% %", v.x, v.y);

    c, d := 7, 8;
    c, d -= 1;
    assert(c == 6 && d == 7, "% %", c, d);
    print("ok\n");
}
