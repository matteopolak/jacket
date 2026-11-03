#import "Basic";

flip :: (x: int) -> int {
    return -x;
}

main :: () {
    print("%\n", flip(5));
}
