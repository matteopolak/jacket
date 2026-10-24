#import "Basic";

check :: (flags: u32, n: int) -> int {
    count := 0;

    if (flags & 4) == 4 count += 1;

    if (flags & 6) == 2 count += 1;

    if (flags & 4) == 0 count += 1;

    if (flags | 2) == 3 count += 1;

    if (flags | 1) != 0x11 count += 1;

    if (n & flags) == 1 count += 1;

    return count;
}

main :: () {
    print("%\n", check(5, 6));
}
