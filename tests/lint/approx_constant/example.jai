#import "Basic";

area :: (r: float) -> float {
    return 3.14159 * r * r;
}

main :: () {
    print("%\n", area(2));
}
