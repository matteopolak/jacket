/// Desktop DPI scale relative to 96 DPI. Window sizes and input coordinates remain device pixels.
//
// ```jai,no_run
// window := create_window(800, 600, "scale");
// scale := get_pixel_scale(window); // metadata for UI/font sizing; dimensions are already pixels
// ```
get_pixel_scale :: (hwnd: Windows.HWND) -> float {
    if !hwnd return 1;
    return dpi_scale(cast(float) window_dpi(hwnd));
}

// Where the window's drawing area is on the screen and how big it is: `x, y, width, height, success`.
//
// - right_handed: counts `y` up from the bottom of the screen instead of down from the top.
//
// ```jai,no_run
// window := create_window(800, 600, "size");
// x, y, width, height, ok := get_dimensions(window, false);
// assert(ok && width > 0);
// ```
get_dimensions :: (hwnd: Windows.HWND, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true) {
    rect: Windows.RECT;
    origin: Windows.POINT;

    if Windows.GetClientRect(hwnd, *rect) == 0 return 0, 0, 0, 0, false;
    if Windows.ClientToScreen(hwnd, *origin) == 0 return 0, 0, 0, 0, false;

    width := rect.right - rect.left;
    height := rect.bottom - rect.top;
    y := origin.y;

    if right_handed y = desktop_bottom() - y - height;

    return origin.x, y, width, height, true;
}

// Where the mouse is, in window coordinates: `x, y, success`.
//
// - right_handed: counts `y` up from the bottom of the window instead of down from the top.
//
// ```jai,no_run
// window := create_window(800, 600, "pointer");
// x, y, ok := get_mouse_pointer_position(window, false);
// ```
get_mouse_pointer_position :: (hwnd: Windows.HWND, right_handed: bool) -> (x: int, y: int, success: bool) {
    pointer: Windows.POINT;
    if Windows.GetCursorPos(*pointer) == 0 return 0, 0, false;
    if Windows.ScreenToClient(hwnd, *pointer) == 0 return 0, 0, false;

    if right_handed {
        rect: Windows.RECT;
        if Windows.GetClientRect(hwnd, *rect) == 0 return 0, 0, false;
        pointer.y = rect.bottom - rect.top - 1 - pointer.y;
    }

    return pointer.x, pointer.y, true;
}

// Where the mouse is, in window coordinates: `x, y, success`.
//
// - right_handed: counts `y` up from the bottom of the window instead of down from the top.
//
// ```jai,no_run
// x, y, ok := get_mouse_pointer_position();
// ```
get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool {
    pointer: Windows.POINT;
    if Windows.GetCursorPos(*pointer) == 0 return 0, 0, false;
    if right_handed pointer.y = desktop_bottom() - 1 - pointer.y;
    return pointer.x, pointer.y, true;
}

#scope_module
desktop_bottom :: () -> s32 {
    return Windows.GetSystemMetrics(Windows.SM_YVIRTUALSCREEN) + Windows.GetSystemMetrics(Windows.SM_CYVIRTUALSCREEN);
}

#scope_module
window_dpi :: (window: Windows.HWND) -> u32 #foreign user32 "GetDpiForWindow";
