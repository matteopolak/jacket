// rules: poly.19
// A struct without parameters has nothing for `#modify` to change.
Foo :: struct #modify {
    return false;
} {
    a: int;
}

main :: () {
    f: Foo;
}
