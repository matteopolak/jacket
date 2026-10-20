#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

bump :: (p: Point) {
    p.x += 10;
}

main :: () {
    bump(.{});
}
