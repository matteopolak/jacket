#import "Basic";

main :: () {
    x := 50;
    a := max(0, min(100, x));
    b := min(100, max(x, 0));
    c := clamp(x, 0, 100);

    print("% % %\n", a, b, c);
}

lowest :: (a: int, b: int) -> int {
    if a < b return a;
    return b;
}

highest :: (a: int, b: int) -> int {
    if a > b return a;
    return b;
}

unnoted_extrema :: (n: int) -> int {
    return highest(0, lowest(100, n));
}
