#import "Basic";

within :: (x: float, limit: float) -> bool {
    return x >= limit;
}

main :: () {
    print("%\n", within(1, 2));
}
