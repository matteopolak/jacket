// rules: num.16 num.17
// An untyped literal operand takes the other operand's type, not the type of the context around
// the operator.
#import "Basic";

take :: (x: float32) -> float32 {
    return x;
}

main :: () {
    w: u16 = 15;
    f: float32 = 0 - w;
    assert(f == 65521);
    assert(take(0 - w) == 65521);
    d: float64 = 1 - w;
    assert(d == 65522);
    // A typed left operand gives its type to a literal on the right as before.
    e: float32 = w - 16;
    assert(e == 65535);
    // Both untyped: folded, then matched to the declaration.
    k: float32 = 2 * 3;
    assert(k == 6);
    u: u8 = 200 + 55;
    assert(u == 255);
    // Mixed with a float operand, the literal is a float as before.
    s: float32 = 1.5;
    assert(1 - s == -0.5);
    // A cast's operand follows the same rule.
    assert(cast(float32) (0 - w) == 65521);
    print("ok\n");
}
