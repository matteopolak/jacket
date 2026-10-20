#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

set :: (p: Point) {
    p = .{1, 2};
}

main :: () {
    set(.{});
}
