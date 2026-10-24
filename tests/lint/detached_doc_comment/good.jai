// Notes on this file as a whole, before any code.

#import "Basic";

// Adds one to the number it is given.
bump :: (n: int) -> int {
    return n + 1;
}

// Helpers

Pair :: struct {
    a: int;
    b: int;
}

// total :: (p: Pair) -> int { return 0; }

weight :: (p: Pair) -> int {
    return p.a + p.b;
}

// TODO: add more directions to this list, such as up and down.

Turn :: enum {
    LEFT;
    RIGHT;
}

main :: () {
    print("%\n", bump(weight(.{ 1, 2 })));
}
