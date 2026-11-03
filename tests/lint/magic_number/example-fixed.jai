#import "Basic";

SECONDS_PER_DAY :: 86400;

expires :: (start: int) -> int {
    return start + SECONDS_PER_DAY;
}

main :: () {
    print("%\n", expires(0));
}
