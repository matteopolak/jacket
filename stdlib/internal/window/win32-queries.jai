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
