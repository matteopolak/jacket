#import "Basic";

Style :: struct {
    count: int;
    color: int;
}

draw :: (count: int, using style: Style) -> int {
    return count + color;
}

main :: () {
    print("%\n", draw(1, .{ 2, 3 }));
}
