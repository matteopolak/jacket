#scope_module
Window_Hook :: struct {
    window: W.HWND;
    previous: s64;
    enabled: bool;
}

window_hooks: [..] Window_Hook;

install_window_hook :: (window: W.HWND) -> *Window_Hook {
    for *hook: window_hooks if hook.window == window return hook;
    if !window return null;

    previous := set_window_long(window, -4, cast(s64) desktop_window_proc);
    if !previous return null;
    if !window_hooks.allocator.proc window_hooks.allocator = context.default_allocator;

    array_add(*window_hooks, .{ window, previous, false });

    return *window_hooks[window_hooks.count - 1];
}

remove_window_hook :: (window: W.HWND) {
    for window_hooks if it.window == window {
        set_window_long(window, -4, it.previous);
        remove it;
        break;
    }

    if !window_hooks.count {
        array_free(window_hooks);
        window_hooks = .{};
    }
}

pump_windows :: () {
    I.input_per_frame_event_and_flag_update();
    message: W.MSG;
    while W.PeekMessageW(*message, null, 0, 0, W.PM_REMOVE) != 0 {
        if message.message == W.WM_QUIT {
            if !I.events_this_frame.allocator.proc I.events_this_frame.allocator = context.default_allocator;
            array_add(*I.events_this_frame, .{ type = .QUIT });
            continue;
        }

        if translate_accelerator(*message) continue;

        W.TranslateMessage(*message);
        W.DispatchMessageW(*message);
    }
}

desktop_window_proc :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s64 #c_call {
    push_context, defer_pop;
    previous: s64;
    enabled := false;

    for hook: window_hooks if hook.window == window {
        previous = hook.previous;
        enabled = hook.enabled;
        break;
    }

    if message == 0x0111 && menu_command(window, wparam, lparam) return 0;

    if enabled && message == W.WM_IME_COMPOSITION {
        if (lparam & 0x800) != 0 queue_composition(window, read_text(window, 0x800), true, false);

        if (lparam & (8 | 0x80)) != 0 {
            text := read_text(window, 8);
            queue_composition(window, text, false, false, cursor_offset(window, text));
        }

        return 0;
    }

    if enabled && message == W.WM_IME_ENDCOMPOSITION {
        queue_composition(window, "", false, true);
        return 0;
    }

    if message == W.WM_NCDESTROY remove_window_hook(window);

    return call_window_proc(previous, window, message, wparam, lparam);
}

set_window_long :: (window: W.HWND, index: s32, value: s64) -> s64 #foreign user32 "SetWindowLongPtrW";
call_window_proc :: (previous: s64, window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s64 #foreign user32 "CallWindowProcW";
