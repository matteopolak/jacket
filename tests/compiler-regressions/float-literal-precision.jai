// Reviewed decimal metadata counts fractional digits, ignoring trailing zeroes.
// Integer-side digits do not select float64. Explicit casts permit rounding.
// rules: num.3 num.18
#import "Basic";

PI_LONG :: 3.14159265358979;

main :: () {
    a := 12342345234.0;
    b := 2.25193;
    c := 16777216.0;
    d := 1234567.0;
    e := 0.12345670;
    huge := 1.0e39;
    tiny := 1.0e-39;
    eight := 0.12345678;
    nine := 0.123456789;
    contextual: float32 = 0.12345678;

    #assert type_of(a) == float32;
    #assert type_of(b) == float32;
    #assert type_of(c) == float32;
    #assert type_of(d) == float32;
    #assert type_of(e) == float32; // trailing zeros do not count
    #assert type_of(huge) == float64;
    #assert type_of(tiny) == float64;
    #assert type_of(PI_LONG) == float64;
    #assert type_of(eight) == float64;
    #assert type_of(nine) == float64;
    #assert type_of(contextual) == float32;
    f: float32 = 2;
    g := f * cast(float32) PI_LONG;

    #assert type_of(g) == float32;
    h: float32 = cast(float32) PI_LONG;

    // Check the represented value independently of Basic's formatting policy.
    assert(cast(float64) a == 12342345728.0);
    assert(c == 16777216.0);

    big: float64 = 1e20;
    rounded := 1e20;

    assert(cast(float64) rounded > big);
    assert(tprint("%", h) == "3.141593");
    assert(tprint("%", cast(float32) 1.0e39) == "Inf"); // an explicit cast rounds to infinity

    print("ok\n");
}
