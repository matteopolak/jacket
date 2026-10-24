#import "Basic";

draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8) -> int {
    return x + y + w + h + r + g + b + a;
}

generic :: (a: $T, b: T, c: T, d: T, e: T, f: T, g: T, h := 0) -> T {
    return a + b + c + d + e + f + g;
}

main :: () {
    callback := (a: int, b: int, c: int, d: int, e: int, f: int, g: int, h: int) -> int {
        return a + b + c + d + e + f + g + h;
    };

    print("% % %\n", draw(1, 2, 3, 4, 5, 6, 7, 8), generic(1, 2, 3, 4, 5, 6, 7), callback(1, 2, 3, 4, 5, 6, 7, 8));
}
