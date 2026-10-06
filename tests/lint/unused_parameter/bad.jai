#import "Basic";

scale :: (value: int, factor: int, label: string) -> int {
    return value * factor;
}

main :: () {
    print("%\n", scale(2, 3, "six"));
}
