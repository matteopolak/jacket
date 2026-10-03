set_custom_cursor_handling :: (is_custom: bool) {
    if mac_custom_cursor == is_custom return;

    mac_custom_cursor = is_custom;

    if !is_custom {
        for window: disabled_cursor_windows {
            Cocoa.void_message(window, "enableCursorRects");
            Cocoa.release(window);
        }

        Basic.array_reset(*disabled_cursor_windows);
    } else {
        pool := Cocoa.autorelease_pool();

        defer Cocoa.release(pool);
        windows := Cocoa.application_windows();
        count := Cocoa.result_message(windows, "count", u64);

        for index: 0..count - 1 {
            window := Cocoa.argument_result_message(windows, "objectAtIndex:", cast(u64) index, Window_Type);
            disable_mac_cursor_rects(window);
        }
    }
}

update_window_events :: () {
    input_per_frame_event_and_flag_update();
    if !Cocoa.initialize_application() return;
    if !install_window_notifications() return;

    pool := Cocoa.autorelease_pool();

    defer Cocoa.release(pool);
    for event: osx_externally_generated_events Basic.array_add(*events_this_frame, event);

    Basic.array_reset_keeping_memory(*osx_externally_generated_events);
    drain_mac_notifications();

    while true {
        native := Cocoa.next_event();
        if !native break;

        translate_mac_event(native);
        Cocoa.dispatch_event(native);
        drain_mac_notifications();
    }

    Cocoa.update_windows();
    drain_mac_notifications();
}

#scope_module
Cocoa :: #import "internal/window/cocoa";
O :: #import "Objective_C";

mac_custom_cursor := false;
window_observer: *void;
window_observer_class: O.Class;
disabled_cursor_windows: [..] Window_Type;
pending_notifications: [..] *void;

install_window_notifications :: () -> bool {
    if window_observer return true;

    superclass := O.objc_getClass("NSObject");

    if !window_observer_class {
        cls := O.objc_allocateClassPair(superclass, "CleanroomWindowInputObserver", 0);
        if !cls {
            observer_error("native window input observer class allocation failed");
            return false;
        }

        added := O.class_addMethod(cls, O.selector("windowNotification:"), cast(O.IMP) mac_window_notification, "v@:@");
        if added == 0 {
            O.objc_disposeClassPair(cls);
            observer_error("native window notification method registration failed");
            return false;
        }

        O.objc_registerClassPair(cls);
        window_observer_class = cls;
    }

    candidate := Cocoa.result_message(window_observer_class, "new", *void);
    if !candidate {
        observer_error("native window observer allocation failed");
        return false;
    }

    center := O.NSNotificationCenter.defaultCenter();
    if !center {
        Cocoa.release(candidate);
        observer_error("native notification center is unavailable");
        return false;
    }

    for name: string.["NSWindowDidResizeNotification", "NSWindowDidMoveNotification", "NSWindowDidBecomeKeyNotification", "NSWindowDidResignKeyNotification", "NSWindowWillCloseNotification", "NSWindowDidMiniaturizeNotification", "NSWindowDidDeminiaturizeNotification"] {
        O.NSNotificationCenter.addObserver(center, cast(O.id) candidate, O.selector("windowNotification:"), cast(*O.NSString) Cocoa.native_string(name), null);
    }

    window_observer = candidate;

    return true;
}

observer_error :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    Basic.assert(false, message);
}

mac_window_notification :: (observer: *void, selector: *void, notification: *void) #c_call {
    push_context, defer_pop;
    Cocoa.void_message(notification, "retain");
    Basic.array_add(*pending_notifications, notification);
}

drain_mac_notifications :: () {
    index := 0;
    while index < pending_notifications.count {
        notification := pending_notifications[index];
        index += 1;
        process_mac_notification(notification);
        Cocoa.release(notification);
    }

    Basic.array_reset_keeping_memory(*pending_notifications);
}

process_mac_notification :: (notification: *void) {
    name := O.to_string(cast(*O.NSString) Cocoa.result_message(notification, "name", *void));
    window := Cocoa.result_message(notification, "object", Window_Type);

    if name == "NSWindowDidResizeNotification" {
        size := Cocoa.content_size(window);
        add_resize_record(window, cast(s32) size.width, cast(s32) size.height);
    } else if name == "NSWindowDidMoveNotification" {
        rect := Cocoa.content_screen_rect(window);
        Basic.array_add(*move_records, Window_Move_Record.{ window, cast(s32) rect.origin.x, cast(s32) rect.origin.y });
    } else if name == "NSWindowDidMiniaturizeNotification" {
        window_minimized = true;
    } else if name == "NSWindowDidDeminiaturizeNotification" {
        window_minimized = false;
    } else {
        event: Event;
        event.window = window;

        if name == "NSWindowWillCloseNotification" {
            event.type = .QUIT;

            retired_index: int = -1;

            for item, index: disabled_cursor_windows {
                if item == window {
                    retired_index = index;
                    break;
                }
            }

            if retired_index >= 0 {
                Basic.array_unordered_remove_by_index(*disabled_cursor_windows, retired_index);
                Cocoa.release(window);
            }
        } else {
            event.type = .WINDOW;
            input_application_has_focus = name == "NSWindowDidBecomeKeyNotification";
            event.window_event_type = .LOST_FOCUS;
            if input_application_has_focus event.window_event_type = .GAINED_FOCUS;
        }

        Basic.array_add(*events_this_frame, event);

        if event.window_event_type == .LOST_FOCUS {
            for state, index: input_button_states {
                if (state & .DOWN) != .NONE emit_mac_button(window, cast(Key_Code) index, false, false, .{});
            }
        }
    }
}

mac_modifiers :: (native: *void) -> Event.Modifier_Flags {
    bits := Cocoa.result_message(native, "modifierFlags", u64);
    result: Event.Modifier_Flags;
    result.shift_pressed = (bits & (1 << 17)) != 0;
    result.ctrl_pressed = (bits & (1 << 18)) != 0;
    result.alt_pressed = (bits & (1 << 19)) != 0;
    result.cmd_meta_pressed = (bits & (1 << 20)) != 0;

    return result;
}

translate_mac_event :: (native: *void) {
    type := Cocoa.result_message(native, "type", u64);
    window := Cocoa.result_message(native, "window", Window_Type);
    flags := mac_modifiers(native);

    if type == 10 || type == 11 {
        code := mac_key_code(native);
        repeat := Cocoa.result_message(native, "isARepeat", s8) != 0;

        text := Cocoa.result_message(native, "characters", *void);

        if code != .UNKNOWN {
            emit_mac_button(window, code, type == 10, repeat, flags);
            events_this_frame[events_this_frame.count - 1].text_input_count = cast(u16) mac_character_count(text);
        }

        if type == 10 emit_mac_characters(window, text, repeat, flags);
    } else if type == 12 {
        sync_mac_modifier(window, .SHIFT, flags.shift_pressed, flags);
        sync_mac_modifier(window, .CTRL, flags.ctrl_pressed, flags);
        sync_mac_modifier(window, .ALT, flags.alt_pressed, flags);
        sync_mac_modifier(window, .CMD, flags.cmd_meta_pressed, flags);
    } else if type == 1 || type == 2 || type == 3 || type == 4 || type == 25 || type == 26 {
        code := Key_Code.MOUSE_BUTTON_LEFT;
        if type == 3 || type == 4 code = .MOUSE_BUTTON_RIGHT;

        if type == 25 || type == 26 {
            button := Cocoa.result_message(native, "buttonNumber", s64);
            if button == 2 code = .MOUSE_BUTTON_MIDDLE;
            else if button == 3 code = .MOUSE_BUTTON_EXTRA_1;
            else if button == 4 code = .MOUSE_BUTTON_EXTRA_2;
            else return;
        }

        emit_mac_button(window, code, type == 1 || type == 3 || type == 25, false, flags);
    } else if type == 5 || type == 6 || type == 7 || type == 27 {
        // Carry fractional native motion across events, never invent integer steps.
        mac_motion_x += Cocoa.result_message(native, "deltaX", float64);
        mac_motion_y += Cocoa.result_message(native, "deltaY", float64);
        dx := cast(int) mac_motion_x;
        dy := cast(int) mac_motion_y;

        mouse_delta_x += dx;
        mouse_delta_y += dy;
        mac_motion_x -= cast(float64) dx;
        mac_motion_y -= cast(float64) dy;
    } else if type == 22 {
        mac_scroll_y += Cocoa.result_message(native, "scrollingDeltaY", float64);
        mac_scroll_x += Cocoa.result_message(native, "scrollingDeltaX", float64);
        dy := cast(s32) mac_scroll_y;
        dx := cast(s32) mac_scroll_x;

        mac_scroll_y -= cast(float64) dy;
        mac_scroll_x -= cast(float64) dx;
        if dy != 0 emit_mac_wheel(window, dy, false, flags);
        if dx != 0 emit_mac_wheel(window, dx, true, flags);
    }

    if mac_custom_cursor && window {
        disable_mac_cursor_rects(window);
    }
}

disable_mac_cursor_rects :: (window: Window_Type) {
    for item: disabled_cursor_windows if item == window return;

    Cocoa.void_message(window, "disableCursorRects");
    Cocoa.result_message(window, "retain", *void);
    if !disabled_cursor_windows.allocator.proc disabled_cursor_windows.allocator = context.default_allocator;

    Basic.array_add(*disabled_cursor_windows, window);
}

mac_motion_x, mac_motion_y: float64;
mac_scroll_x, mac_scroll_y: float64;

emit_mac_wheel :: (window: Window_Type, delta: s32, horizontal: bool, flags: Event.Modifier_Flags) {
    event: Event;
    event.type = .MOUSE_WHEEL;
    event.window = window;
    event.modifier_flags = flags;
    event.typical_wheel_delta = 1;
    event.wheel_delta = delta;

    if !horizontal {
        mouse_delta_z += delta;
        if delta > 0 event.key_code = .MOUSE_WHEEL_UP;
        else event.key_code = .MOUSE_WHEEL_DOWN;
    }

    Basic.array_add(*events_this_frame, event);
}

sync_mac_modifier :: (window: Window_Type, code: Key_Code, down: bool, flags: Event.Modifier_Flags) {
    was_down := (input_button_states[cast(int) code] & .DOWN) != .NONE;
    if was_down != down emit_mac_button(window, code, down, false, flags);
}

emit_mac_button :: (window: Window_Type, code: Key_Code, down: bool, repeat: bool, flags: Event.Modifier_Flags) {
    event: Event;
    event.type = .KEYBOARD;
    event.window = window;
    event.key_code = code;
    event.key_pressed = cast(u32) down;
    event.repeat = repeat;
    event.modifier_flags = flags;
    state := *input_button_states[cast(int) code];

    if down {
        state.* |= .DOWN;
        if !repeat state.* |= .START;
    } else state.* = (state.* & ~Key_Current_State.DOWN) | .END;

    Basic.array_add(*events_this_frame, event);
}

emit_mac_characters :: (window: Window_Type, text: *void, repeat: bool, flags: Event.Modifier_Flags) {
    count := Cocoa.result_message(text, "length", u64);
    index: u64 = 0;

    while index < count {
        unit := cast(u32) Cocoa.argument_result_message(text, "characterAtIndex:", index, u16);

        index += 1;

        if unit >= 0xd800 && unit <= 0xdbff {
            if index < count {
                low := cast(u32) Cocoa.argument_result_message(text, "characterAtIndex:", index, u16);
                if low >= 0xdc00 && low <= 0xdfff {
                    unit = 0x10000 + ((unit - 0xd800) << 10) + low - 0xdc00;
                    index += 1;
                } else unit = 0xfffd;
            } else unit = 0xfffd;
        } else if unit >= 0xdc00 && unit <= 0xdfff unit = 0xfffd;

        // AppKit private-use function-key characters are keys, not text.
        if unit <= 31 || unit == 127 || (unit >= 0xf700 && unit <= 0xf8ff) continue;

        event: Event;
        event.type = .TEXT_INPUT;
        event.window = window;
        event.utf32 = unit;
        event.repeat = repeat;
        event.modifier_flags = flags;
        Basic.array_add(*events_this_frame, event);
    }
}

mac_character_count :: (text: *void) -> u64 {
    length := Cocoa.result_message(text, "length", u64);
    count: u64;
    index: u64;

    while index < length {
        unit := Cocoa.argument_result_message(text, "characterAtIndex:", index, u16);

        index += 1;
        if unit >= 0xd800 && unit <= 0xdbff && index < length {
            next := Cocoa.argument_result_message(text, "characterAtIndex:", index, u16);
            if next >= 0xdc00 && next <= 0xdfff index += 1;
        }

        count += 1;
    }

    return count;
}

mac_key_code :: (native: *void) -> Key_Code {
    physical := Cocoa.result_message(native, "keyCode", u16);
    for item: mac_special_keys if item.native == physical return item.code;

    characters := Cocoa.result_message(native, "charactersIgnoringModifiers", *void);
    if Cocoa.result_message(characters, "length", u64) == 0 return .UNKNOWN;

    unit := cast(u32) Cocoa.argument_result_message(characters, "characterAtIndex:", cast(u64) 0, u16);
    if unit >= #char "a" && unit <= #char "z" unit -= 32;
    if unit >= 32 && unit <= 126 return cast(Key_Code) unit;

    return .UNKNOWN;
}

Mac_Special_Key :: struct {
    native: u16;
    code: Key_Code;
}

// Public Apple HIToolbox Events.h layout-independent virtual key codes.
mac_special_keys := Mac_Special_Key.[
    .{ 0x24, .ENTER }, .{ 0x30, .TAB }, .{ 0x31, .SPACEBAR }, .{ 0x33, .BACKSPACE }, .{ 0x35, .ESCAPE },
    .{ 0x36, .CMD }, .{ 0x37, .CMD }, .{ 0x38, .SHIFT }, .{ 0x3a, .ALT }, .{ 0x3b, .CTRL },
    .{ 0x3c, .SHIFT }, .{ 0x3d, .ALT }, .{ 0x3e, .CTRL }, .{ 0x73, .HOME }, .{ 0x74, .PAGE_UP },
    .{ 0x75, .DELETE }, .{ 0x77, .END }, .{ 0x79, .PAGE_DOWN }, .{ 0x7b, .ARROW_LEFT }, .{ 0x7c, .ARROW_RIGHT },
    .{ 0x7d, .ARROW_DOWN }, .{ 0x7e, .ARROW_UP }, .{ 0x7a, .F1 }, .{ 0x78, .F2 }, .{ 0x63, .F3 },
    .{ 0x76, .F4 }, .{ 0x60, .F5 }, .{ 0x61, .F6 }, .{ 0x62, .F7 }, .{ 0x64, .F8 }, .{ 0x65, .F9 },
    .{ 0x6d, .F10 }, .{ 0x67, .F11 }, .{ 0x6f, .F12 }, .{ 0x69, .F13 }, .{ 0x6b, .F14 }, .{ 0x71, .F15 },
    .{ 0x6a, .F16 }, .{ 0x40, .F17 }, .{ 0x4f, .F18 }, .{ 0x50, .F19 }, .{ 0x5a, .F20 }
];
