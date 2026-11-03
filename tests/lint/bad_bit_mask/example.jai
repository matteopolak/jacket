#import "Basic";

can_write :: (flags: u32) -> bool {
    return (flags & 4) == 1;
}

main :: () {
    print("%\n", can_write(5));
}
