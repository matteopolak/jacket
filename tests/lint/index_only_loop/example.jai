#import "Basic";

sum :: (xs: [] int) -> int {
    total := 0;

    for i: 0..xs.count - 1 {
        total += xs[i];
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
