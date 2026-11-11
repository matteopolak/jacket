// A `#run` inside a procedure can read the constants of a `using` parameter's struct type, its
// struct parameters among them, as it can through `b.N`. Its fields stay out of reach.
// rules: proc.18
#import "Basic";

Box :: struct(N: int, types: ..Type) {
    x: int;
    LIMIT :: 4;
}

has :: (using b: *Box, $T: Type) -> bool {
    found :: #run -> bool {
        for types if it == T return true;
        return false;
    };

    #run assert(N == 3 && LIMIT == 4);

    return found;
}

main :: () {
    b: Box(3, string, float);
    assert(has(*b, string));
    assert(!has(*b, int));
}
