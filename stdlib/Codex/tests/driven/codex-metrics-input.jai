global_value: s64;

original_leaf :: (value: s64) -> s64 {
    return value + 1;
}

original_linear :: (value: s64) -> s64 {
    local := original_leaf(value);
    global_value = local;
    local += global_value;
    return original_leaf(local);
}

original_loops :: () {
    for 0..2 {
        counter := 2;
        while counter > 0 {
            counter -= 1;
        }
    }

    for 0..1 {}
}

main :: () {}
