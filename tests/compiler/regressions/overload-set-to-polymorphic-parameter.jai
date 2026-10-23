// An overload set passed for a polymorphic parameter binds the member with the declared procedure
// type, or the first member when the parameter's type is a bare `$F`.
#import "Basic";

add :: (a: int, b: int) -> int {
    return a + b;
}

add :: (a: float, b: float) -> float {
    return a * b;
}

typed_baked_int :: ($f: (int, int) -> int, x: int) -> int {
    return f(x, x);
}

typed_baked_float :: ($f: (float, float) -> float, x: float) -> float {
    return f(x, x);
}

untyped_baked :: ($f: $F, x: int) -> int {
    return f(x, x);
}

untyped_runtime :: (f: $F, x: int) -> int {
    return f(x, x);
}

main :: () {
    assert(typed_baked_int(add, 3) == 6);
    assert(typed_baked_float(add, 3) == 9);
    assert(untyped_baked(add, 3) == 6);
    assert(untyped_runtime(add, 3) == 6);
    print("ok\n");
}
