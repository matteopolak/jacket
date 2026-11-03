#import "Basic";

size_in_bytes :: (count: int) -> int {
    return count * 1;
}

main :: () {
    print("%\n", size_in_bytes(8));
}
