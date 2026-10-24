#import "Basic";

sum :: (data: *u8, n: s64, small: u32) -> int {
    total := 0;

    for i: 0..n - 1 {
        total += data[i];
    }

    for 0..small {
        total += 1;
    }

    return total;
}

guarded :: (count: u32) {
    if count > 0 {
        for 0..count - 2 {
            print("%\n", it);
        }
    }

    for 0..4 - 1 {
        print("%\n", it);
    }
}

main :: () {
    bytes: [4] u8;
    print("%\n", sum(bytes.data, 4, 2));
    guarded(3);
}
