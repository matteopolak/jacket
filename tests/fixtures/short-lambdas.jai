apply :: (value: int, callback: (int) -> int) -> int {
    return callback(value);
}

main :: () -> int {
    OFFSET :: 2;
    typed := (value: int) => value + OFFSET;
    contextual: (int) -> int = (value) => value + OFFSET;
    return typed(10) + contextual(10) + apply(16, (value) => value + OFFSET);
}
