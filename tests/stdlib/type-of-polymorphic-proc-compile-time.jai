#import "Basic";

// type_of / type_info of a polymorphic procedure is allowed at compile time.
poly :: (x: $T) -> T {
    return x * x;
}
concrete :: (x: int) -> int {
    return x * x;
}

#run {
    t := type_of(poly);
    u := type_of(concrete);
    assert(t != u);
    info := type_info(t);
    assert(info != null);
}

main :: () {
    assert(poly(3) == 9);
}
