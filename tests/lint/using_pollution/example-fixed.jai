#import "Basic";

Style :: struct {
    count: int;
    color: int;
}

draw :: (count: int, style: Style) -> int {
    return count + style.color;
}

main :: () {
    print("%\n", draw(1, .{ 2, 3 }));
}
