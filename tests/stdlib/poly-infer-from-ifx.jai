// An `ifx` argument binds a polymorphic parameter through its branches (found by tools/jaigen.py).
#import "Basic";

combine :: (a: $T, b: T) -> T {
    return a * b + 1;
}

main :: () {
    x: s16 = 300;
    y: s16 = 7;
    flag := x > y;
    r := combine(ifx flag then x else y, y);
    assert(type_of(r) == s16 && r == 2101);
    // Parenthesized, and with only an untyped branch on one side.
    q := combine((ifx flag then y else 2), y);
    assert(type_of(q) == s16 && q == 50);
    // Both branches untyped: the literal default, as for a plain literal argument.
    n := combine(ifx flag then 3 else 4, 5);
    assert(type_of(n) == s64 && n == 16);
    f: float32 = 0.5;
    g := combine(ifx flag then 2 else f, f);
    assert(type_of(g) == float32 && g == 2);
    print("ok\n");
}
