#import "Basic";

square :: (n: int) -> int {
    result := n * n;
    return result;
}

main :: () {
    print("%\n", square(4));
}
