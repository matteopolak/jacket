#import "Basic";

// Adds one to the number it is given.

bump :: (n: int) -> int {
    return n + 1;
}

Pair :: struct {
    a: int;
    b: int;
}

/* Counts the pieces of a pair
   and returns them as one number. */

weight :: (p: Pair) -> int {
    return p.a + p.b;
}

// The directions a robot can turn.
// Left and right are relative to its heading.

Turn :: enum {
    LEFT;
    RIGHT;
}

main :: () {
    print("%\n", bump(weight(.{ 1, 2 })));
}
