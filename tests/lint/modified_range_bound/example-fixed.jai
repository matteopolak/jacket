#import "Basic";

count_nonzero :: (xs: [] int) -> int {
    n := xs.count;

    for xs {
        if it == 0 n -= 1;
    }

    return n;
}

main :: () {
    print("%\n", count_nonzero(.[1, 0, 2]));
}
