#import "Basic";

sum :: (data: *u8, n: s64) -> int {
    total := 0;

    for i: 0..n - 1 {
        total += data[i];
    }

    return total;
}

main :: () {
    bytes := u8.[1, 2, 3];
    print("%\n", sum(bytes.data, 3));
}
