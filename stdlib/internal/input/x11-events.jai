set_custom_cursor_handling :: (is_custom: bool) {
    // X11 retains the caller-selected window cursor; this pump never overrides it.
}

update_window_events :: () {
    input_per_frame_event_and_flag_update();
    display := Connection.get_display();
    if !display return;

    while XPending(display) > 0 {
        native: XEvent;

        XNextEvent(display, *native);
        window := native.xany.window;
        if !x_window_is_ours(display, window) continue;

        flags := x11_modifiers(0);

        if native.type == KeyPress || native.type == KeyRelease {
            if native.type == KeyRelease && XPending(display) > 0 {
                next: XEvent;
                XPeekEvent(display, *next);
                if next.type == KeyPress && next.xkey.time == native.xkey.time && next.xkey.keycode == native.xkey.keycode continue;
            }

            bytes: [64] u8;
            symbol: KeySym;
            count := XLookupString(*native.xkey, bytes.data, 64, *symbol, null);
            code := get_key_code(symbol);

            flags = x11_modifiers(native.xkey.state);
            repeat := (input_button_states[cast(int) code] & .DOWN) != .NONE && native.type == KeyPress;
            if code != .UNKNOWN x11_button(window, code, native.type == KeyPress, repeat, flags);

            if native.type == KeyPress {
                for index: 0..count - 1 {
                    if bytes[index] <= 31 || bytes[index] == 127 continue;

                    event: Event;
                    event.type = .TEXT_INPUT;
                    event.window = window;
                    event.utf32 = bytes[index];
                    event.modifier_flags = flags;
                    Basic.array_add(*events_this_frame, event);
                }
            }
        } else if native.type == ButtonPress || native.type == ButtonRelease {
            button := native.xbutton.button;

            flags = x11_modifiers(native.xbutton.state);

            if button >= 4 && button <= 7 {
                if native.type != ButtonPress continue;

                event: Event;
                event.type = .MOUSE_WHEEL;
                event.window = window;
                event.modifier_flags = flags;
                event.typical_wheel_delta = WHEEL_DELTA;
                event.wheel_delta = WHEEL_DELTA;
                if button == 5 || button == 6 event.wheel_delta = -WHEEL_DELTA;

                if button <= 5 {
                    mouse_delta_z += event.wheel_delta;
                    event.key_code = .MOUSE_WHEEL_UP;
                    if button == 5 event.key_code = .MOUSE_WHEEL_DOWN;
                }

                Basic.array_add(*events_this_frame, event);
            } else {
                code := Key_Code.UNKNOWN;
                if button == 1 code = .MOUSE_BUTTON_LEFT;
                if button == 2 code = .MOUSE_BUTTON_MIDDLE;
                if button == 3 code = .MOUSE_BUTTON_RIGHT;
                if button == 8 code = .MOUSE_BUTTON_EXTRA_1;
                if button == 9 code = .MOUSE_BUTTON_EXTRA_2;
                if code != .UNKNOWN x11_button(window, code, native.type == ButtonPress, false, flags);
            }
        } else if native.type == MotionNotify {
            dx, dy := Connection.pointer_motion(window, native.xmotion.x, native.xmotion.y);
            mouse_delta_x += dx;
            mouse_delta_y += dy;
        } else if native.type == ConfigureNotify {
            add_resize_record(window, native.xconfigure.width, native.xconfigure.height);
            Basic.array_add(*move_records, Window_Move_Record.{ window, native.xconfigure.x, native.xconfigure.y });
        } else if native.type == FocusIn || native.type == FocusOut {
            input_application_has_focus = native.type == FocusIn;
            event: Event;
            event.type = .WINDOW;
            event.window = window;
            event.window_event_type = .LOST_FOCUS;
            if input_application_has_focus event.window_event_type = .GAINED_FOCUS;

            Basic.array_add(*events_this_frame, event);

            if !input_application_has_focus for state, index: input_button_states {
                if (state & .DOWN) != .NONE x11_button(window, cast(Key_Code) index, false, false, .{});
            }
        } else if native.type == ClientMessage {
            protocols := XInternAtom(display, "WM_PROTOCOLS", 0);
            close_atom := XInternAtom(display, "WM_DELETE_WINDOW", 0);

            if native.xclient.message_type == protocols && native.xclient.format == 32 && native.xclient.data.l[0] == cast(s64) close_atom {
                event: Event;
                event.type = .QUIT;
                event.window = window;
                Basic.array_add(*events_this_frame, event);
            }
        } else if native.type == DestroyNotify Connection.release_window(window);
    }
}

#scope_module
#import "X11";
Connection :: #import "internal/window/connection";

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
    Basic.array_add(*events_this_frame, event);
}
