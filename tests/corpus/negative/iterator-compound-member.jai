#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    points: [..] Point;
    array_add(*points, .{1, 2});
    for points it.x += 1;
}
