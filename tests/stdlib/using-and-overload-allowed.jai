// A direct member may repeat a promoted name, `using,except` leaves a name out, overloads
// with different parameter types are fine, and so are polymorphic ones with the same shape.
A :: struct {
    x: int;
}

B :: struct {
    x: float;
    z: int;
}

C :: struct {
    using a: A;
    using,except(x) b: B;
}

D :: struct {
    using a: A;
    x: float;
}

f :: (a: int) -> int {
    return 1;
}

f :: (a: float) -> int {
    return 2;
}

g :: (a: $T) -> int {
    return 3;
}

g :: (a: $U, b: int) -> int {
    return 4;
}

main :: () -> s64 {
    c: C;
    d: D;
    c.x = 1;
    c.z = 2;
    d.x = 1.5;
    if f(1) != 1 || f(1.0) != 2 || g("s") != 3 return 1;
    return 0;
}
