// A call with one result cannot fill two names.
single :: (x: int) -> int {
    return x;
}

main :: () {
    a, b := single(1);
}
