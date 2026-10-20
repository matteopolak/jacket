#import "Basic";
W :: #import "Windows";
I :: #import "Input";
WC :: #import "Window_Creation";

user32 :: #library,system "user32";
post_message :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s32 #foreign user32 "PostMessageW";

main :: () {
    win := WC.create_window(800, 600, "Issue #1 native wheel regression");

    assert(win != null);
    defer assert(W.DestroyWindow(win) != 0);
    I.update_window_events();

    deltas := s32.[-32768, -120, -1, 0, 1, 120, 32767];

    for message: u32.[W.WM_MOUSEWHEEL, W.WM_MOUSEHWHEEL] {
        for delta: deltas {
            encoded := delta;
            if encoded < 0 encoded += 65536;

            // Nonzero upper pointer-width bits and every native mouse-key bit.
            native_word: u64 = cast(u64) encoded;
            packed := 0x123456780000007f | (native_word << 16);

            assert((packed >> 32) == 0x12345678);

            // A genuine mouse move (Windows sends one when a window appears under the cursor) is
            // queued as input, so it is handled after the posted wheel in the same pump and sets
            // the held keys from its own bits. Post again until a frame keeps the wheel's keys.
            keys_held := false;

            for 1..8 {
                assert(post_message(win, message, packed, 0x8000ffff) != 0);
                I.update_window_events();

                // The window shares the desktop with the machine's real input, so a frame may also hold
                // focus changes or a genuine wheel notch. Pick out the posted message by its content
                // and check the frame's accumulated wheel total against the vertical wheel events.
                ours := 0;
                vertical_total := 0;

                for event: I.events_this_frame {
                    if event.type != .MOUSE_WHEEL || event.window != win continue;
                    if event.key_code == .MOUSE_WHEEL_UP || event.key_code == .MOUSE_WHEEL_DOWN vertical_total += event.wheel_delta;
                    if event.wheel_delta == delta && event.typical_wheel_delta == 120 && event.shift_pressed && event.ctrl_pressed ours += 1;
                }

                assert(ours == 1);
                assert(I.mouse_delta_z == vertical_total);

                keys_held = (I.input_button_states[cast(int) I.Key_Code.SHIFT] & .DOWN) != .NONE
                    && (I.input_button_states[cast(int) I.Key_Code.CTRL] & .DOWN) != .NONE;
                if keys_held break;
            }

            assert(keys_held);
        }
    }

    // The original scroll-down encoding must survive repeated frame pumps. Real input may add
    // other events to a frame, so count only this window's wheel events.
    for 0..255 {
        assert(post_message(win, W.WM_MOUSEWHEEL, 0xff880000, 0) != 0);
        I.update_window_events();

        ours := 0;
        vertical_total := 0;

        for event: I.events_this_frame {
            if event.type != .MOUSE_WHEEL || event.window != win continue;
            if event.key_code == .MOUSE_WHEEL_UP || event.key_code == .MOUSE_WHEEL_DOWN vertical_total += event.wheel_delta;
            if event.wheel_delta == -120 ours += 1;
        }

        assert(ours >= 1);
        assert(I.mouse_delta_z == vertical_total);
    }

    assert(post_message(win, W.WM_MOVE, 0, 0x123456788000ffff) != 0);
    I.update_window_events();
    moves := I.get_window_moves();

    assert(moves.count == 1);
    assert(moves[0].window == win && moves[0].x == -1 && moves[0].y == -32768);

    print("ok\n");
}
