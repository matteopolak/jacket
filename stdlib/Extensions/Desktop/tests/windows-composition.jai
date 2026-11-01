// test: only(.WINDOWS)
#load "../module.jai";

previous_proc :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s64 #c_call {
    return 1234;
}

main :: () {
    begin_desktop_events();
    array_add(*window_hooks, .{ null, cast(s64) previous_proc, true });
    assert(desktop_window_proc(null, W.WM_IME_ENDCOMPOSITION, 0, 0) == 0);
    assert(composition_events.count == 1 && composition_events[0].cancel);
    assert(I.events_this_frame.count == 0);
    window_hooks[0].enabled = false;
    assert(desktop_window_proc(null, W.WM_IME_COMPOSITION, 0, 0) == 1234);
    array_free(window_hooks);
    array_free(composition_events);
}
