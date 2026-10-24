#import "Basic";

inside :: (x: float, limit: float, n: int, flag: bool) -> bool {
    if x >= limit return false;

    // Integers have no NaN.
    if !(n < 10) return false;

    // `==` negates exactly into `!=`.
    if !(x == limit) return false;

    return !flag;
}

main :: () {
    print("%\n", inside(1, 2, 3, true));
}
