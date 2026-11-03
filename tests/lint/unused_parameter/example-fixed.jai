#import "Basic";

area_of :: (value: int, factor: int) -> int {
    return value * factor;
}

main :: () {
    print("%\n", area_of(2, 3));
}
