// `operator* :: #procedure_of_call f(...)` adds an existing procedure to the operator's overloads,
// next to ordinary `operator*` definitions.
#import "Basic";

V :: struct {
    x: float;
}

operator* :: (a: V, b: V) -> V {
    return .{ a.x * b.x };
}

scale :: (a: V, s: float) -> V {
    return .{ a.x * s };
}

operator* :: #procedure_of_call scale(V.{}, 1.0);

main :: () {
    a := V.{ 3 };
    assert((a * V.{ 2 }).x == 6);
    assert((a * 4.0).x == 12);

    print("ok\n");
}
