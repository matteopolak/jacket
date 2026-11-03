#import "Basic";

main :: () {
    x := 50;
    a := min(0, max(100, x));
    b := max(100, min(x, 0));
    print("% %\n", a, b);
}

// Body inference recognizes extrema independently of names or notes.
smallest :: (a: int, b: int) -> int {
    if a < b return a;
    return b;
}

largest :: (a: int, b: int) -> int {
    if a > b return a;
    return b;
}

noted_extrema :: (n: int) -> int {
    return smallest(0, largest(100, n));
}
