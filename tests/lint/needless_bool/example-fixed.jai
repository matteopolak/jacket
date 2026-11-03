#import "Basic";

is_positive :: (n: int) -> bool {
    return n > 0;
}

main :: () {
    print("%\n", is_positive(3));
}
