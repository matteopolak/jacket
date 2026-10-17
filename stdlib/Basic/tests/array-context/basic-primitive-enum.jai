B :: #import "Basic";

Kind :: enum {
    FIRST;
    SECOND;
}

check :: () -> bool {
    ints := int.[1, 2, 3];
    enums := Kind.[.FIRST, .SECOND];
    a, ai := B.array_find(ints, 2);
    b, bi := B.array_find(enums, Kind.SECOND);

    return a && ai == 1 && b && bi == 1;
}

#assert check();

main :: () {
    B.assert(check());
}
