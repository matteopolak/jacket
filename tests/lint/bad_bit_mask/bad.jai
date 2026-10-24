#import "Basic";

check :: (flags: u32, n: int) -> int {
    count := 0;

    if (flags & 4) == 1 count += 1;

    if (flags | 2) == 0 count += 1;

    if 8 != (flags & 3) count += 1;

    if (n | 1) != 0x10 count += 1;

    return count;
}

main :: () {
    print("%\n", check(5, 6));
}
