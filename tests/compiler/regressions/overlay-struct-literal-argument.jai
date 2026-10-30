// Overlay members take no positional value, so a `.{...}` argument fills the members that own
// storage and the positions after an overlay do not shift.
// rules: struct.25
#import "Basic";
#import "Math";

Rect :: struct {
    x: float;
    y: float;
    #overlay (x) pos: Vector2;

    w: float;
    h: float;
    #overlay (w) size: Vector2;
}

width_of :: (rect: Rect) -> float {
    return rect.w;
}

main :: () {
    ww: s32 = 640;
    hh: s32 = 480;

    assert(width_of(.{ 0, 0, xx ww, xx hh }) == 640);
    assert(width_of(.{ 1, 2, 3, 4 }) == 3);
    print("ok\n");
}
