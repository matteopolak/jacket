// An untyped struct literal next to an operator with several overloads: the literal's values
// choose between them, and compound assignment and comparisons resolve the same way.
#import "Basic";

S :: struct {
    a, b: float;
}

T :: struct {
    a, b, c: float;
}

// Two values fit `S`; `T` has three fields without initializers, so a two-value literal is not a `T`.
operator * :: (x: S, f: float) -> S {
    return .{ x.a * f, x.b * f };
}

operator * :: (x: T, f: float) -> T {
    return .{ x.a * f, x.b * f, x.c * f };
}

operator + :: (x: S, y: S) -> S {
    return .{ x.a + y.a, x.b + y.b };
}

operator == :: (x: S, y: S) -> bool {
    return x.a == y.a && x.b == y.b;
}

operator < :: (x: S, y: S) -> bool {
    return x.a < y.a;
}

main :: () {
    scaled := .{ 1.0, 2.0 } * 2.0;

    assert(type_of(scaled) == S);
    assert(scaled.a == 2 && scaled.b == 4);

    triple := .{ 1.0, 2.0, 3.0 } * 2.0;

    assert(type_of(triple) == T);
    assert(triple.c == 6);

    v := S.{ 1, 2 };
    v += .{ 10, 20 };

    assert(v.a == 11 && v.b == 22);

    assert(v == .{ 11, 22 });
    assert(.{ 11, 22 } == v);
    assert(v != .{ 1, 2 });
    assert(!(v != .{ 11, 22 }));
    assert(.{ 1, 0 } < v);
    assert(!(v < .{ 1, 0 }));

    print("ok\n");
}
