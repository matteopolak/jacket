#import "Basic";

flip :: (x: int) -> int {
    return x * -1;
}

main :: () {
    print("%\n", flip(5));
}
