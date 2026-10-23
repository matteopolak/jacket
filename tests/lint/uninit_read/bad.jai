#import "Basic";

Point :: struct {
    x, y: int;
}

sum :: (xs: [] int) -> int {
    total: int = ---;
    for xs total += it;
    return total;
}

first :: () -> u8 {
    buffer: [64] u8 = ---;
    return buffer[0];
}

corner :: () -> int {
    p: Point = ---;
    print("%\n", p.x);
    return 0;
}

main :: () {
    print("%\n", sum(.[1, 2]));
    print("%\n", first());
    print("%\n", corner());
}
