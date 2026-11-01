// A `using` of a variable that clashes with several constants of the block. The clash that is
// reported must not depend on how the compiler numbers the spellings.
Point :: struct {
    zulu: int;
    mike: int;
    alpha: int;
    quebec: int;
    bravo: int;
    tango: int;
}

main :: () {
    p: Point;
    zulu :: 1;
    mike :: 2;
    alpha :: 3;
    quebec :: 4;
    bravo :: 5;
    tango :: 6;

    using p;
}
