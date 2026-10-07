// A module parameter without a written type keeps an untyped argument untyped, so it
// passes to unsigned and smaller parameters (ui_builder's Pixel_Maker hands
// `MAX_FRAME_IN_FLIGHT := 3` to an `NSUInteger`).
// rules: modparam.8
#import "Basic";

FRAME_COUNT :: 2;

Frames :: #import "Untyped_Param"(FRAMES = FRAME_COUNT);

main :: () {
    assert(Frames.frames_as_count() == 2);
    assert(Frames.limit_as_byte() == 7);
    small: u16 = Frames.FRAMES;
    assert(small == 2);
    print("ok\n");
}
