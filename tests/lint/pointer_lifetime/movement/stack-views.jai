bad_fixed_view :: () -> [] int {
    a: [2] int;
    return a;
}

good_persistent_literal :: () -> [] int {
    return int.[1, 2];
}
