#import "Basic";

Rect :: struct {
    x, y, w, h: int;
}

Color :: struct {
    r, g, b, a: u8;
}

draw :: (rect: Rect, color: Color) -> int {
    return rect.x + rect.y + rect.w + rect.h + color.r + color.g + color.b + color.a;
}

main :: () {
    print("%\n", draw(.{ 0, 0, 10, 10 }, .{ 255, 0, 0, 255 }));
}
