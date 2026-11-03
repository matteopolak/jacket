#import "Basic";

sum :: (items: [] int) -> int {
    total := 0;
    for items total += it;
    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
