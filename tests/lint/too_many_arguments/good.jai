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

// Exactly at the limit.
seven :: (a: int, b: int, c: int, d: int, e: int, f: int, g: int) -> int {
    return a + b + c + d + e + f + g;
}

// A foreign-style signature is fixed by C.
eight :: (a: s32, b: s32, c: s32, d: s32, e: s32, f: s32, g: s32, h: s32) -> s32 #c_call {
    return a + b + c + d + e + f + g + h;
}

main :: () {
    print("% %\n", draw(.{ 1, 2, 3, 4 }, .{ 5, 6, 7, 8 }), seven(1, 2, 3, 4, 5, 6, 7));
}
