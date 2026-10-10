// Two parameters of one procedure cannot share a name.
add :: (value: int, value: int) -> int {
    return value;
}

main :: () {
    add(1, 2);
}
