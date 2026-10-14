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
            packed := 0x123456780000007f | (cast(u64) encoded << 16);

            assert(post_message(win, message, packed, 0x8000ffff) != 0);
            I.update_window_events();
            assert(I.events_this_frame.count == 1);
            e := I.events_this_frame[0];

            assert(e.type == .MOUSE_WHEEL && e.window == win);
            assert(e.wheel_delta == delta && e.typical_wheel_delta == 120);
            assert(e.shift_pressed && e.ctrl_pressed);
            expected_z := 0;
            if message == W.WM_MOUSEWHEEL expected_z = delta;

            assert(I.mouse_delta_z == expected_z);
            assert((I.input_button_states[cast(int) I.Key_Code.SHIFT] & .DOWN) != .NONE);
            assert((I.input_button_states[cast(int) I.Key_Code.CTRL] & .DOWN) != .NONE);
        }
    }

    // The original scroll-down encoding must survive repeated frame pumps.
    for 0..255 {
        assert(post_message(win, W.WM_MOUSEWHEEL, 0xff880000, 0) != 0);
        I.update_window_events();
        assert(I.events_this_frame.count == 1);
        assert(I.events_this_frame[0].wheel_delta == -120);
        assert(I.mouse_delta_z == -120);
    }

    assert(post_message(win, W.WM_MOVE, 0, 0x123456788000ffff) != 0);
    I.update_window_events();
    moves := I.get_window_moves();

    assert(moves.count == 1);
    assert(moves[0].window == win && moves[0].x == -1 && moves[0].y == -32768);

    print("ok\n");
}
