#import "Basic";

Vec :: struct {
    x: int;
    y: int;
}

same :: (a: Vec, b: Vec) -> bool {
    return a.x == b.x && a.y == b.y;
}

main :: () {
    print("%\n", same(.{ 1, 2 }, .{ 1, 3 }));
}
