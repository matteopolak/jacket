#import "Basic";

scale :: (a: int) -> int {
    total := a * 2;

    return total;
}

main :: () {
    print("%\n", scale(4));
}
