#import "Basic";

sum_above :: (items: [] int, limit: int) -> int {
    total := 0;

    for items {
        if it > limit && it >= 0 total += it;
    }

    return total;
}

main :: () {
    print("%\n", sum_above(.[3, -1, 8], 2));
}
