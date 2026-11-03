#import "Basic";

// Adds one to the number it is given.
bump :: (n: int) -> int {
    return n + 1;
}

main :: () {
    print("%\n", bump(1));
}
