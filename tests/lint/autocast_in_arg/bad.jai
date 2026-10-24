#import "Basic";

draw :: (x: s32, y: float) -> s32 {
    return x + cast(s32) y;
}

main :: () {
    width: s64 = 640;
    scale: float64 = 1.5;

    print("%\n", draw(xx width, xx scale));
    print("%\n", draw(1, y = xx scale));
}
