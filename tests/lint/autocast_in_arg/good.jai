#import "Basic";

draw :: (x: s32, y: float) -> s32 {
    return x + cast(s32) y;
}

main :: () {
    width: s64 = 640;
    scale: float64 = 1.5;

    print("%\n", draw(cast(s32) width, cast(float) scale));

    // Not an argument: `autocast_used` looks at these.
    small: s32 = xx width;

    print("%\n", draw(small, 1));
}
