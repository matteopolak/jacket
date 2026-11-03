#import "Basic";

over_limit :: (count: int, limit: int) -> bool {
    return count > limit;
}

main :: () {
    print("%\n", over_limit(5, 4));
}
