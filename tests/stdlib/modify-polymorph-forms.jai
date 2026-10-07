// rules: poly.18 poly.20
// `#modify` on the procedures that have polymorph variables: a `$R` only in the return type
// that the block binds, a baked `$n`, and `$$` parameters (the block runs for calls that bake).
#import "Basic";

widen :: (x: $T) -> $R #modify { R = float64; } { return cast(R) x; }

positive :: ($n: int) -> int #modify { return n > 0, "n must be positive"; } { return n; }

// `n` is visible to the block only in the instances that baked it; `twice(v)` does not run it.
twice :: ($$n: int) -> int #modify { return n < 100, "n is too large to bake"; } { return n * 2; }

main :: () {
    w := widen(5);
    assert(type_of(w) == float64 && w == 5.0);
    assert(positive(3) == 3);
    v := 3;
    assert(twice(4) == 8);
    assert(twice(v) == 6);
    print("ok\n");
}
