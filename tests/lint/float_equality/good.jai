#import "Basic";
#import "Math";

Pair :: struct {
    x: float;
    y: float;
}

operator == :: (p: Pair, q: Pair) -> bool {
    return p.x == q.x && p.y == q.y;
}

main :: () {
    a := 0.1;
    b := 0.2;
    total := a + b;
    // Against a constant: an exact stored value.
    if total == 0 print("zero\n");
    // The NaN test.
    if total != total print("not a number\n");
    if abs(total - 0.3) < 0.0001 print("close enough\n");
    if Pair.{ a, b } == Pair.{ b, a } print("same pair\n");
}
