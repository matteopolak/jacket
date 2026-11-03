#import "Basic";

sum :: (xs: [] int) -> int {
    total := 0;

    for xs {
        total += it;
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
