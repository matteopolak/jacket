#import "Basic";

main :: () {
    x := 50;
    a := min(0, max(100, x));
    b := max(100, min(x, 0));
    print("% %\n", a, b);
}

// The notes, not the names `min` and `max`, make these the library's extrema.
smallest :: (a: int, b: int) -> int {
    if a < b return a;
    return b;
} @Minimum

largest :: (a: int, b: int) -> int {
    if a > b return a;
    return b;
} @Maximum

noted_extrema :: (n: int) -> int {
    return smallest(0, largest(100, n));
}
