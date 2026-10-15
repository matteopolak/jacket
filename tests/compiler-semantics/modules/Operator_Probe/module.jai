Scalar :: struct {
    value: int;
}

operator - :: (value: Scalar) -> Scalar {
    return .{ -value.value };
}

operator - :: (left: Scalar, right: Scalar) -> Scalar {
    return .{ left.value - right.value };
}
