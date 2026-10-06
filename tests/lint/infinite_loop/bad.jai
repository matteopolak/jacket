#import "Basic";

sum :: (values: [] int) -> int {
    total := 0;
    i := 0;
    while i < values.count {
        total += values[i];
    }
    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
