set_custom_cursor_handling :: (is_custom: bool) {
    custom_cursor = is_custom;
}

// A program that handles the cursor itself keeps whatever cursor it set on a window. Otherwise
// the window's cursor goes back to the default as the pointer moves over it, as on the other
// platforms.
custom_cursor := false;

update_window_events :: () {
    input_per_frame_event_and_flag_update();
    display := Connection.get_display();
    if !display return;

    while XPending(display) > 0 {
        native: XEvent;

        XNextEvent(display, *native);
        window := native.xany.window;
        if !x_window_is_ours(display, window) continue;

        if native.type == KeyPress || native.type == KeyRelease {
            x11_handle_key(display, window, *native);
        } else if native.type == ButtonPress || native.type == ButtonRelease {
            x11_handle_mouse_button(window, *native);
        } else if native.type == MotionNotify {
            x11_handle_motion(display, window, *native);
        } else if native.type == ConfigureNotify {
            add_resize_record(window, native.xconfigure.width, native.xconfigure.height);
            add_to_frame(*move_records, Window_Move_Record.{ window, native.xconfigure.x, native.xconfigure.y });
        } else if native.type == FocusIn || native.type == FocusOut {
            x11_handle_focus(window, native.type == FocusIn);
        } else if native.type == ClientMessage {
            x11_handle_client_message(display, window, *native);
        } else if native.type == DestroyNotify Connection.release_window(window);
    }
}

#scope_module
#import "X11";
Connection :: #import "internal/window/connection";

x11_handle_key :: (display: *Display, window: Window, native: *XEvent) {
    // An auto-repeat arrives as a release immediately followed by an identical press; drop the pair.
    if native.type == KeyRelease && XPending(display) > 0 {
        next: XEvent;
        XPeekEvent(display, *next);
        if next.type == KeyPress && next.xkey.time == native.xkey.time && next.xkey.keycode == native.xkey.keycode return;
    }

    bytes: [64] u8;
    symbol: KeySym;
    count := XLookupString(*native.xkey, bytes.data, 64, *symbol, null);
    code := get_key_code(symbol);

    flags := x11_modifiers(native.xkey.state);
    repeat := (input_button_states[cast(int) code] & .DOWN) != .NONE && native.type == KeyPress;

    if code != .UNKNOWN x11_button(window, code, native.type == KeyPress, repeat, flags);

    if native.type == KeyPress x11_text_input(window, bytes.data, count, flags);
}

x11_text_input :: (window: Window, bytes: *u8, count: s32, flags: Event.Modifier_Flags) {
    for index: 0..count - 1 {
        if bytes[index] <= 31 || bytes[index] == 127 continue;

        event: Event;
        event.type = .TEXT_INPUT;
        event.window = window;
        event.utf32 = bytes[index];
        event.modifier_flags = flags;
        add_to_frame(*events_this_frame, event);
    }
}

x11_handle_mouse_button :: (window: Window, native: *XEvent) {
    button := native.xbutton.button;
    flags := x11_modifiers(native.xbutton.state);

    if button >= 4 && button <= 7 {
        if native.type == ButtonPress x11_wheel(window, button, flags);
        return;
    }

    code := Key_Code.UNKNOWN;
    if button == 1 code = .MOUSE_BUTTON_LEFT;
    if button == 2 code = .MOUSE_BUTTON_MIDDLE;
    if button == 3 code = .MOUSE_BUTTON_RIGHT;
    if button == X11_BUTTON_EXTRA_1 code = .MOUSE_BUTTON_EXTRA_1;
    if button == X11_BUTTON_EXTRA_2 code = .MOUSE_BUTTON_EXTRA_2;
    if code != .UNKNOWN x11_button(window, code, native.type == ButtonPress, false, flags);
}

x11_wheel :: (window: Window, button: u32, flags: Event.Modifier_Flags) {
    event: Event;
    event.type = .MOUSE_WHEEL;
    event.window = window;
    event.modifier_flags = flags;
    event.typical_wheel_delta = WHEEL_DELTA;
    event.wheel_delta = WHEEL_DELTA;
    if button == X11_WHEEL_DOWN || button == X11_WHEEL_LEFT event.wheel_delta = -WHEEL_DELTA;

    if button <= X11_WHEEL_DOWN {
        mouse_delta_z += event.wheel_delta;
        event.key_code = .MOUSE_WHEEL_UP;
        if button == X11_WHEEL_DOWN event.key_code = .MOUSE_WHEEL_DOWN;
    }

    add_to_frame(*events_this_frame, event);
}

x11_handle_motion :: (display: *Display, window: Window, native: *XEvent) {
    if !custom_cursor XUndefineCursor(display, window);

    dx, dy := Connection.pointer_motion(window, native.xmotion.x, native.xmotion.y);
    mouse_delta_x += dx;
    mouse_delta_y += dy;
}

x11_handle_focus :: (window: Window, gained: bool) {
    input_application_has_focus = gained;
    event: Event;
    event.type = .WINDOW;
    event.window = window;
    event.window_event_type = .LOST_FOCUS;
    if gained event.window_event_type = .GAINED_FOCUS;

    add_to_frame(*events_this_frame, event);

    if !gained for state, index: input_button_states {
        if (state & .DOWN) != .NONE x11_button(window, cast(Key_Code) index, false, false, .{});
    }
}

x11_handle_client_message :: (display: *Display, window: Window, native: *XEvent) {
    protocols := XInternAtom(display, "WM_PROTOCOLS", 0);
    close_atom := XInternAtom(display, "WM_DELETE_WINDOW", 0);

    if native.xclient.message_type == protocols && native.xclient.format == 32 && native.xclient.data.l[0] == cast(s64) close_atom {
        event: Event;
        event.type = .QUIT;
        event.window = window;
        add_to_frame(*events_this_frame, event);
    }
}

x11_modifiers :: (state: u32) -> Event.Modifier_Flags {
    flags: Event.Modifier_Flags;
    flags.shift_pressed = (state & ShiftMask) != 0;
    flags.ctrl_pressed = (state & ControlMask) != 0;
    flags.alt_pressed = (state & Mod1Mask) != 0;
    flags.cmd_meta_pressed = (state & Mod4Mask) != 0;

    return flags;
}

x11_button :: (window: Window, code: Key_Code, down: bool, repeat: bool, flags: Event.Modifier_Flags) {
    state := *input_button_states[cast(int) code];

    if down {
        state.* |= .DOWN;
        if !repeat state.* |= .START;
    } else state.* = (state.* & ~Key_Current_State.DOWN) | .END;

    event: Event;
    event.type = .KEYBOARD;
    event.window = window;
    event.key_code = code;
    event.key_pressed = cast(u32) down;
    event.repeat = repeat;
    event.modifier_flags = flags;
    add_to_frame(*events_this_frame, event);
}

#scope_file

// Core X11 pointer button numbers past the three standard buttons.
X11_WHEEL_DOWN :: 5;
X11_WHEEL_LEFT :: 6;
X11_BUTTON_EXTRA_1 :: 8;
X11_BUTTON_EXTRA_2 :: 9;
