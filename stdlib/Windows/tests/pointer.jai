// test: only(.WINDOWS)
#import "Basic";
W :: #import "Windows";
I :: #import "Input";
WC :: #import "Window_Creation";

user32 :: #library,system "user32";
post :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s32 #foreign user32 "PostMessageW";

motion :: (window: W.HWND, x: s32, y: s32, flags: u64 = 0) {
    if x < 0 x += 65536;
    if y < 0 y += 65536;

    wide_x: u64 = cast(u64) x;
    wide_y: u64 = cast(u64) y;
    packed := 0x1234567800000000 | wide_x | (wide_y << 16);

    assert(post(window, W.WM_MOUSEMOVE, flags, cast(s64) packed) != 0);
}

// Whether the last pump delivered a focus change. Tests run in parallel on one desktop, so another
// process's window can take the foreground at any moment; the library then correctly releases every
// held key and forgets the pointer position, which is not what this test checks.
focus_changed :: () -> bool {
    for event: I.events_this_frame {
        if event.type == .WINDOW && (event.window_event_type == .LOST_FOCUS || event.window_event_type == .GAINED_FOCUS) return true;
    }

    return false;
}

main :: () {
    first := WC.create_window(200, 100, "Native pointer regression one");
    second := WC.create_window(200, 100, "Native pointer regression two");

    assert(first != null && second != null);
    defer assert(W.DestroyWindow(second) != 0);
    defer assert(W.DestroyWindow(first) != 0);

    // Redo the whole sequence when a foreign focus change interrupted it; a real regression fails every attempt.
    for 1..50 {
        if scenario(first, second) {
            print("ok\n");
            return;
        }
    }

    assert(false, "every attempt was interrupted by a focus change");
}

scenario :: (first: W.HWND, second: W.HWND) -> bool {
    I.update_window_events();
    // Reset each native client-coordinate baseline through the actual pump.
    assert(post(first, W.WM_MOVE, 0, 0) != 0);
    assert(post(second, W.WM_MOVE, 0, 0) != 0);
    motion(first, 10, 20);
    motion(first, 13, 15, 0x7f);
    motion(second, -32768, 32767, 0x7f);
    motion(second, 32767, -32768, 0x7f);
    I.update_window_events();
    if focus_changed() return false;

    assert(I.mouse_delta_x == 3 + 65535 && I.mouse_delta_y == -5 - 65535);
    assert((I.input_button_states[cast(int) I.Key_Code.SHIFT] & .DOWN) != .NONE);
    assert((I.input_button_states[cast(int) I.Key_Code.CTRL] & .DOWN) != .NONE);
    assert((I.input_button_states[cast(int) I.Key_Code.MOUSE_BUTTON_EXTRA_2] & .DOWN) != .NONE);
    motion(first, -1, -2);
    I.update_window_events();
    if focus_changed() return false;

    assert(I.mouse_delta_x == -14 && I.mouse_delta_y == -17);
    assert((I.input_button_states[cast(int) I.Key_Code.SHIFT] & .DOWN) == .NONE);
    assert((I.input_button_states[cast(int) I.Key_Code.MOUSE_BUTTON_EXTRA_2] & .DOWN) == .NONE);
    assert(post(first, W.WM_KILLFOCUS, 0, 0) != 0);
    motion(first, -32768, 32767);
    I.update_window_events();
    assert(I.mouse_delta_x == 0 && I.mouse_delta_y == 0);
    motion(first, -32767, 32766);
    I.update_window_events();
    if focus_changed() return false;

    assert(I.mouse_delta_x == 1 && I.mouse_delta_y == -1);
    I.update_window_events();
    assert(I.mouse_delta_x == 0 && I.mouse_delta_y == 0);
    assert(post(first, W.WM_LBUTTONDOWN, 0x000d, 0) != 0);
    I.update_window_events();
    if focus_changed() return false;

    found := false;

    for event: I.events_this_frame if event.type == .KEYBOARD && event.key_code == .MOUSE_BUTTON_LEFT {
        assert(event.key_pressed == 1 && event.shift_pressed && event.ctrl_pressed);
        found = true;
    }

    assert(found && (I.input_button_states[cast(int) I.Key_Code.MOUSE_BUTTON_LEFT] & .START) != .NONE);
    assert(post(first, W.WM_LBUTTONUP, 0, 0) != 0);
    I.update_window_events();
    if focus_changed() return false;

    found = false;

    for event: I.events_this_frame if event.type == .KEYBOARD && event.key_code == .MOUSE_BUTTON_LEFT {
        assert(event.key_pressed == 0 && !event.shift_pressed && !event.ctrl_pressed);
        found = true;
    }

    assert(found && I.input_button_states[cast(int) I.Key_Code.MOUSE_BUTTON_LEFT] == .END);

    return true;
}
