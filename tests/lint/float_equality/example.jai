#import "Basic";

same_total :: (a: float, b: float, expected: float) -> bool {
    return a + b == expected;
}

main :: () {
    print("%\n", same_total(0.1, 0.2, 0.3));
}
