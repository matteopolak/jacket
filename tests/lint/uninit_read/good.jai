#import "Basic";

Point :: struct {
    x, y: int;
}

fill :: (p: *Point) {
    p.x = 1;
    p.y = 2;
}

sum :: (xs: [] int) -> int {
    total: int = ---;
    total = 0;
    for xs total += it;
    return total;
}

filled :: () -> int {
    p: Point = ---;
    fill(*p);
    return p.x;
}

zeroed :: () -> u8 {
    buffer: [64] u8 = ---;
    memset(buffer.data, 0, buffer.count);
    return buffer[0];
}

by_pointer :: () -> int {
    values: [4] int = ---;
    for *values it.* = it_index;
    return values[3];
}

branches :: (flag: bool) -> int {
    n: int = ---;

    if flag {
        n = 1;
    } else {
        n = 2;
    }

    return n;
}

sized :: () -> int {
    p: Point = ---;
    return size_of(type_of(p));
}

main :: () {
    print("%\n", sum(.[1, 2]));
    print("%\n", filled());
    print("%\n", zeroed());
    print("%\n", by_pointer());
    print("%\n", branches(true));
    print("%\n", sized());
}
