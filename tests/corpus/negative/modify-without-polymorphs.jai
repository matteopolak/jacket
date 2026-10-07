// rules: poly.19
// A `#modify` block on a procedure with no `$` parameters never ran and was not even checked
// (`print` is not imported); it is now an error at the block.
main :: () {
    foo(5);
}

foo :: (x: int) -> int #modify {
    print("%", x);
    return x > 5;
} {
    return x + 1;
}
