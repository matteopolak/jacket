#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

set :: (p: Point) {
    p.x = 1;
}

main :: () {
    set(.{1, 2});
}
