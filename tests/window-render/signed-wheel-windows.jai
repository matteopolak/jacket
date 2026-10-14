// Issue #1: the high word of WM_MOUSEWHEEL is a signed 16-bit delta.
// This must run through the public native handler and shared event queue.
#assert OS == .WINDOWS, "signed wheel regression requires Windows";

#import "Basic";
I :: #import "Input";
W :: #import "Windows";

main :: () {
    for delta: s32.[120, -120, -240, 30, -30] {
        I.input_per_frame_event_and_flag_update();
        encoded := cast(u64) cast,no_check(u16) delta;
        result := I.MyWindowProc(null, cast(u32) W.WM_MOUSEWHEEL, encoded << 16, 0);

        assert(result == 0);
        assert(I.events_this_frame.count == 1);
        event := I.events_this_frame[0];

        assert(event.type == .MOUSE_WHEEL);
        assert(event.wheel_delta == delta);
        assert(event.typical_wheel_delta == 120);
        assert(event.window == null);
    }

    I.input_per_frame_event_and_flag_update();
    assert(I.events_this_frame.count == 0);
    print("signed wheel propagation ok\n");
}
