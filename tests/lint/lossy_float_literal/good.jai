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

    print("% % % % % % % % % % %\n", a, b, c, d, e, f, g, h, i, j, k);
}
