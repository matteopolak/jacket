indexed :: (values: *s64) {
    values[0] = 42;
}

main :: () {
    value: s64;
    indexed(*value);
}
