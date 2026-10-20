#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    points := Point.[.{1, 2}, .{3, 4}];
    for points it = .{5, 6};
}
