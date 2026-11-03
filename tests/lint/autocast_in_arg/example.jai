#import "Basic";

draw :: (x: s32, y: s32) -> s32 {
    return x + y;
}

main :: () {
    width: s64 = 640;

    print("%\n", draw(xx width, 10));
}
