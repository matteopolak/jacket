#import "Basic";

main :: () {
    // Short enough for a float32.
    a: float32 = 3.1415927;
    b: float32 = 0.1;
    c: float32 = 16777216.0;
    d: float32 = 1.5e10;
    e: float32 = 100000000.0;

    // Trailing zeros carry nothing.
    f: float32 = 0.5000000000000;

    // A float64 keeps all of its digits.
    g: float64 = 12345678.12345678;
    h := 12342345234.0;
    i := g * 123456789.123456789;

    // Converted by a cast or by the callee, not by where it stands.
    j := cast(float64) 123456.789012;
    k := cast(float32) 123456.789012;

    // More fractional digits than a `float32` keeps make the literal a `float64`, which widens
    // the operation it is in; shortening it would change the type.
    x: float32 = 1;
    l := x + 0.1591549431;
    m := 0.1591549431 + x;

    assert(type_of(l) == float64 && type_of(m) == float64);

    print("% % % % % % % % % % % % %\n", a, b, c, d, e, f, g, h, i, j, k, l, m);
}
