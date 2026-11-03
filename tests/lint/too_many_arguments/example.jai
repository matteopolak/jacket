#import "Basic";

draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8) -> int {
    return x + y + w + h + r + g + b + a;
}

main :: () {
    print("%\n", draw(0, 0, 10, 10, 255, 0, 0, 255));
}
