#import "Basic";

sum :: (xs: [] int) -> int {
    total: int = ---;
    for xs total += it;
    return total;
}

main :: () {
    print("%\n", sum(int.[1, 2, 3]));
}
