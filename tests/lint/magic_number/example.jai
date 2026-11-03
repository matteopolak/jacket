#import "Basic";

expires :: (start: int) -> int {
    return start + 86400;
}

main :: () {
    print("%\n", expires(0));
}
