#import "Basic";
#import "Math";

same_total :: (a: float, b: float, expected: float) -> bool {
    return abs(a + b - expected) < 0.000001;
}

main :: () {
    print("%\n", same_total(0.1, 0.2, 0.3));
}
