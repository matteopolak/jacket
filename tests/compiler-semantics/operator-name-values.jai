// Operator declarations can be resolved directly and selected as ordinary procedure values.
#import "Basic";
Probe :: #import "Operator_Probe";

Pair :: struct {
    x: int;
}

operator - :: (value: Pair) -> Pair {
    return .{ -value.x };
}

operator - :: (left: Pair, right: Pair) -> Pair {
    return .{ left.x - right.x };
}

main :: () {
    left := Pair.{ 9 };
    right := Pair.{ 4 };
    negated := -left;
    difference := left - right;
    direct := operator -(left);
    unary: (Pair) -> Pair = operator -;
    binary: (Pair, Pair) -> Pair = operator -;

    assert(negated.x == -9 && difference.x == 5 && direct.x == -9);
    assert(unary(left).x == -9 && binary(left, right).x == 5);
    imported := Probe.Scalar.{7};

    // A namespace import alone does not make its operators lexically visible.
    using Probe;
    assert((-imported).value == -7);
    qualified := Probe.operator -(imported);
    selected: (Probe.Scalar) -> Probe.Scalar = Probe.operator -;

    assert(qualified.value == -7 && selected(imported).value == -7);
}
