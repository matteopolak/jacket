set_custom_cursor_handling :: (is_custom: bool) {
    custom_cursor = is_custom;
}

update_window_events :: () {
    input_per_frame_event_and_flag_update();
    message: MSG;
    while PeekMessageW(*message, null, 0, 0, PM_REMOVE) != 0 {
        if message.message == WM_QUIT {
            event: Event;
            event.type = .QUIT;
            Basic.array_add(*events_this_frame, event);
            continue;
        }

        TranslateMessage(*message);
        DispatchMessageW(*message);
    }
}

MyWindowProc :: (hwnd: HWND, message: u32, wParam: WPARAM, lParam: LPARAM) -> s64 #c_call {
    push_context, defer_pop;

    if message == WM_CREATE {
        if !Devices.open(hwnd) return -1;
        return 0;
    }

    if message == WM_NCDESTROY {
        Devices.close(hwnd);
        forget_pointer(hwnd);
    }

    if message == WM_DEVICECHANGE {
        if Devices.decode(wParam, cast(*DEV_BROADCAST_HDR) lParam, the_gamepad_handler) return 1;
    }

    if message == WM_MOUSEMOVE {
        bits := cast(u64) (lParam & 0xffffffff);
        x := Native_Fields.signed_word(bits);
        y := Native_Fields.high_signed_word(bits);

        record_pointer(hwnd, x, y);
        windows_mouse_modifiers(wParam);

        return 0;
    }

    if message == WM_MOUSEWHEEL || message == WM_MOUSEHWHEEL {
        // Native signed high word, independent of all other WPARAM bits.
        delta := Native_Fields.high_signed_word(wParam);
        event: Event;
        event.type = .MOUSE_WHEEL;
        event.window = hwnd;
        event.wheel_delta = delta;
        event.typical_wheel_delta = 120;
        event.modifier_flags = windows_mouse_modifiers(wParam);

        if message == WM_MOUSEWHEEL {
            mouse_delta_z += delta;
            if delta > 0 event.key_code = .MOUSE_WHEEL_UP;
            if delta < 0 event.key_code = .MOUSE_WHEEL_DOWN;
        }

        // Horizontal messages retain their signed delta. The reviewed Event
        // has no axis field; UNKNOWN avoids misreporting a vertical direction.
        Basic.array_add(*events_this_frame, event);

        return 0;
    }

    if message == WM_KEYDOWN || message == WM_SYSKEYDOWN || message == WM_KEYUP || message == WM_SYSKEYUP {
        down := message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        code := get_key_code(wParam);

        if code != .UNKNOWN emit_windows_button(hwnd, code, down, (lParam & (1 << 30)) != 0 && down);
        // Preserve native system-key actions, such as Alt+F4.
        if message == WM_SYSKEYDOWN || message == WM_SYSKEYUP return DefWindowProcW(hwnd, message, wParam, lParam);

        return 0;
    }

    if message == WM_CHAR {
        if wParam > 31 emit_windows_text(hwnd, cast(u32) wParam);
        return 0;
    }

    if message == WM_SETFOCUS || message == WM_KILLFOCUS {
        input_application_has_focus = message == WM_SETFOCUS;
        event: Event;
        event.type = .WINDOW;
        event.window = hwnd;
        event.window_event_type = .LOST_FOCUS;
        if input_application_has_focus event.window_event_type = .GAINED_FOCUS;

        Basic.array_add(*events_this_frame, event);

        if !input_application_has_focus {
            reset_pointer(hwnd);

            // Release held keys on focus loss; otherwise a missed native key-up
            // would leave the application stuck in a pressed state.
            for state, index: input_button_states {
                if state & .DOWN emit_windows_button(hwnd, cast(Key_Code) index, false, false);
            }

        }

        return 0;
    }

    if message == WM_SIZE {
        window_minimized = wParam == SIZE_MINIMIZED;
        bits := cast(u64) (lParam & 0xffffffff);
        add_resize_record(hwnd, cast(s32) (bits & 0xffff), cast(s32) ((bits >> 16) & 0xffff));
        return 0;
    }

    if message == WM_MOVE {
        reset_pointer(hwnd);
        bits := cast(u64) (lParam & 0xffffffff);
        Basic.array_add(*move_records, Window_Move_Record.{ hwnd, Native_Fields.signed_word(bits), Native_Fields.high_signed_word(bits) });
        return 0;
    }

    if message == WM_DESTROY Window_Lifecycle.notify_closed(hwnd);

    if message == WM_CLOSE {
        event: Event;
        event.type = .QUIT;
        event.window = hwnd;
        Basic.array_add(*events_this_frame, event);

        return 0;
    }

    if message == WM_SETCURSOR && custom_cursor return 1;

    if message == WM_LBUTTONDOWN || message == WM_LBUTTONUP {
        emit_windows_button(hwnd, .MOUSE_BUTTON_LEFT, message == WM_LBUTTONDOWN, false, windows_mouse_modifiers(wParam));
        return 0;
    }

    if message == WM_RBUTTONDOWN || message == WM_RBUTTONUP {
        emit_windows_button(hwnd, .MOUSE_BUTTON_RIGHT, message == WM_RBUTTONDOWN, false, windows_mouse_modifiers(wParam));
        return 0;
    }

    if message == WM_MBUTTONDOWN || message == WM_MBUTTONUP {
        emit_windows_button(hwnd, .MOUSE_BUTTON_MIDDLE, message == WM_MBUTTONDOWN, false, windows_mouse_modifiers(wParam));
        return 0;
    }

    if message == WM_XBUTTONDOWN || message == WM_XBUTTONUP {
        code := Key_Code.MOUSE_BUTTON_EXTRA_1;
        if ((wParam >> 16) & 0xffff) == 2 code = .MOUSE_BUTTON_EXTRA_2;
        emit_windows_button(hwnd, code, message == WM_XBUTTONDOWN, false, windows_mouse_modifiers(wParam));
        return 1;
    }

    return DefWindowProcW(hwnd, message, wParam, lParam);
}

#scope_module
Native_Fields :: #import "internal/input/native-fields";
Window_Lifecycle :: #import "internal/window/lifecycle";
Devices :: #import "internal/input/windows-devices";

custom_cursor := false;

Pointer_Record :: struct {
    window: HWND;
    x, y: s32;
    positioned: bool;
}

pointer_records: [..] Pointer_Record;

record_pointer :: (window: HWND, x: s32, y: s32) {
    for *entry: pointer_records if entry.window == window {
        if entry.positioned {
            mouse_delta_x += cast(int) x - entry.x;
            mouse_delta_y += cast(int) y - entry.y;
        }

        entry.x = x;
        entry.y = y;
        entry.positioned = true;

        return;
    }

    if !pointer_records.allocator.proc pointer_records.allocator = context.default_allocator;

    Basic.array_add(*pointer_records, Pointer_Record.{ window, x, y, true });
}

reset_pointer :: (window: HWND) {
    for *entry: pointer_records if entry.window == window entry.positioned = false;
}

forget_pointer :: (window: HWND) {
    for entry, index: pointer_records if entry.window == window {
        Basic.array_unordered_remove_by_index(*pointer_records, index);
        return;
    }
}

windows_mouse_modifiers :: (packed: WPARAM) -> Event.Modifier_Flags {
    flags := windows_modifiers();
    flags.shift_pressed = (packed & 4) != 0;
    flags.ctrl_pressed = (packed & 8) != 0;
    sync_mouse_key(.SHIFT, flags.shift_pressed);
    sync_mouse_key(.CTRL, flags.ctrl_pressed);
    sync_mouse_key(.MOUSE_BUTTON_LEFT, (packed & 1) != 0);
    sync_mouse_key(.MOUSE_BUTTON_RIGHT, (packed & 2) != 0);
    sync_mouse_key(.MOUSE_BUTTON_MIDDLE, (packed & 16) != 0);
    sync_mouse_key(.MOUSE_BUTTON_EXTRA_1, (packed & 32) != 0);
    sync_mouse_key(.MOUSE_BUTTON_EXTRA_2, (packed & 64) != 0);

    return flags;
}

sync_mouse_key :: (key: Key_Code, down: bool) {
    state := *input_button_states[cast(int) key];
    if down state.* |= .DOWN;
    else state.* &= ~Key_Current_State.DOWN;
}

windows_modifiers :: () -> Event.Modifier_Flags {
    flags: Event.Modifier_Flags;
    flags.shift_pressed = GetKeyState(cast(s32) VK_SHIFT) < 0;
    flags.ctrl_pressed = GetKeyState(cast(s32) VK_CONTROL) < 0;
    flags.alt_pressed = GetKeyState(cast(s32) VK_MENU) < 0;
    flags.cmd_meta_pressed = GetKeyState(cast(s32) VK_LWIN) < 0 || GetKeyState(cast(s32) VK_RWIN) < 0;

    return flags;
}

emit_windows_button :: (window: HWND, key: Key_Code, down: bool, repeat: bool, flags: Event.Modifier_Flags = windows_modifiers()) {
    event: Event;
    event.type = .KEYBOARD;
    event.window = window;
    event.key_code = key;
    event.key_pressed = cast(u32) down;
    event.repeat = repeat;
    event.modifier_flags = flags;
    state := *input_button_states[cast(int) key];

    if down {
        state.* |= .DOWN;
        if !repeat state.* |= .START;
    } else {
        state.* = (state.* & ~Key_Current_State.DOWN) | .END;
    }

    Basic.array_add(*events_this_frame, event);
}

emit_windows_text :: (window: HWND, codepoint: u32) {
    event: Event;
    event.type = .TEXT_INPUT;
    event.window = window;
    event.utf32 = codepoint;
    event.modifier_flags = windows_modifiers();
    Basic.array_add(*events_this_frame, event);
}
