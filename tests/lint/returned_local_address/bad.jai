#import "Basic";

Point :: struct {
    x, y: int;
}

counter :: () -> *int {
    count := 0;
    return *count;
}

corner :: () -> *int {
    p: Point;
    return *p.y;
}

slot :: (i: int) -> *u8 {
    buffer: [64] u8;
    return *buffer[i];
}

scratch :: () -> *void {
    buffer: [64] u8;
    return xx buffer.data;
}

main :: () {
    print("%\n", counter());
    print("%\n", corner());
    print("%\n", slot(3));
    print("%\n", scratch());
}

// A value parameter belongs to this stack frame too.
parameter :: (n: int) -> *int {
    return *n;
}
