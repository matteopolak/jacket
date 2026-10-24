#import "Basic";

sum :: (data: *u8, n: u32) -> int {
    total := 0;

    for i: 0..n - 1 {
        total += data[i];
    }

    return total;
}

pairs :: (count: u64) {
    for 1..count - 1 {
        print("%\n", it);
    }
}

main :: () {
    bytes: [4] u8;
    print("%\n", sum(bytes.data, 4));
    pairs(3);
}
