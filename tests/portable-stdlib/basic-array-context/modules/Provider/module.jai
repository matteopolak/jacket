Marker :: struct {
    value: int;
}

operator == :: (a: Marker, b: Marker) -> bool {
    return a.value == b.value;
}

equal :: (a: $T, b: T) -> bool {
    return a == b;
}
