#import "Basic";

// -----------------------------------------------------------------------------
// Helpers

double :: (n: int) -> int {
    return n * 2;
}

main :: () {
    print("%\n", double(4));
}
