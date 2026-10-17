#import "Basic";
W :: #import "Windows";
WC :: #import "Window_Creation";
S :: #import "Simp";
I :: #import "Input";

user32 :: #library,system "user32";
post_message :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s32 #foreign user32 "PostMessageW";

main :: () {
    window := WC.create_window(800, 600, "Issue #1 production renderer and event pump");

    assert(window != null);
    defer assert(W.DestroyWindow(window) != 0);
    S.set_render_target(window);
    I.update_window_events();
    deltas := s32.[-32768, -120, 0, 120, 32767];

    for message: u32.[W.WM_MOUSEWHEEL, W.WM_MOUSEHWHEEL] {
        for delta: deltas {
            encoded := delta;
            if encoded < 0 encoded += 65536;

            // Native MK_SHIFT (0x4) and MK_CONTROL (0x8), plus unrelated upper bits.
            native_word: u64 = cast(u64) encoded;
            upper: u64 = 0x12345678;
            packed := (native_word << 16) | 0x000c | (upper << 32);

            assert((packed >> 32) == upper);

            assert(post_message(window, message, packed, 0) != 0);
            I.update_window_events();
            found := false;

            for event: I.events_this_frame if event.type == .MOUSE_WHEEL {
                assert(event.window == window && event.wheel_delta == delta);
                assert(event.shift_pressed && event.ctrl_pressed);
                found = true;
            }

            assert(found);
        }
    }

    print("ok\n");
}
