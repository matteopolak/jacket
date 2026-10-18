// A viable non-polymorphic overload beats `$T`, even when the argument needs a conversion
// (widening an integer, dereferencing a pointer to the struct); `$T` still takes what the
// concrete overloads cannot accept.
#import "Basic";

G :: struct {
    n: int;
}

pick :: (it: $T) -> string {
    return "poly";
}

pick :: (g: G) -> string {
    return "G";
}

pick :: (x: float) -> string {
    return "float";
}

pick :: (x: s64) -> string {
    return "s64";
}

main :: () {
    g := New(G);

    assert(pick(g) == "G");
    i: int = 3;

    assert(pick(i) == "s64");
    j: s32 = 3;

    assert(pick(j) == "s64");
    b: u8 = 3;

    assert(pick(b) == "s64");
    d: float64 = 1;

    assert(pick(d) == "poly");
    f: float32 = 1;

    assert(pick(f) == "float");
}
