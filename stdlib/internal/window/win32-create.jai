create_window :: (width: int, height: int, window_name: string, window_x := -1, window_y := -1, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> Windows.HWND {
    if width <= 0 || height <= 0 return null;

    instance := Windows.GetModuleHandleW(null);
    if !instance return null;

    if !window_class_registered {
        color: u32;

        for channel: 0..2 {
            component := cast(u32) (background_color_rgb[channel] * 255);
            Basic.assert(component <= 255, "window background channels must lie in [0, 1]");
            color |= component << (channel * 8);
        }

        brush := Windows.CreateSolidBrush(color);
        if !brush return null;

        wc: Windows.WNDCLASSEXW;
        wc.cbSize = size_of(Windows.WNDCLASSEXW);
        wc.style = Windows.CS_OWNDC | Windows.CS_HREDRAW | Windows.CS_VREDRAW;
        wc.lpfnWndProc = Input.MyWindowProc;
        wc.hInstance = instance;
        wc.hCursor = Windows.LoadCursorW(null, cast(*s16) Windows.IDC_ARROW);
        wc.hbrBackground = brush;
        wc.lpszClassName = window_class_name.data;
        if Windows.RegisterClassExW(*wc) == 0 {
            error := win_get_last_error();

            Basic.assert(Windows.DeleteObject(brush) != 0, "window brush cleanup failed");
            // Module parameter instances share the process-wide native class.
            if error != 1410 return null; // ERROR_CLASS_ALREADY_EXISTS

            existing: Windows.WNDCLASSEXW;
            existing.cbSize = size_of(Windows.WNDCLASSEXW);
            if win_get_class_info(instance, window_class_name.data, *existing) == 0 return null;
            if existing.lpfnWndProc != Input.MyWindowProc || (existing.style & Windows.CS_OWNDC) == 0 return null;
        }

        window_class_registered = true;
    }

    style := WINDOWED_STYLE;
    if parent style = SECONDARY_WINDOW_STYLE;

    rect: Windows.RECT;
    rect.right = cast(s32) width;
    rect.bottom = cast(s32) height;
    if Windows.AdjustWindowRectEx(*rect, style, 0, 0) == 0 return null;

    if window_x == -1 || window_y == -1 {
        work: Windows.RECT;
        if Windows.SystemParametersInfoW(Windows.SPI_GETWORKAREA, 0, *work, 0) == 0 return null;
        if window_x == -1 window_x = work.left;
        if window_y == -1 window_y = work.top;
    }

    title := window_utf16(window_name);

    defer Basic.free(title);
    if !title return null;

    window := Windows.CreateWindowExW(0, window_class_name.data, title, style, cast(s32) window_x, cast(s32) window_y, rect.right - rect.left, rect.bottom - rect.top, parent, null, instance, null);
    if !window return null;

    Windows.ShowWindow(window, Windows.SW_SHOW);

    return window;
}

toggle_fullscreen :: (hwnd: Windows.HWND, desire_fullscreen: bool, saved: *Saved_Window_Info) -> (succcess: bool, width: s32, height: s32) {
    if !hwnd || !saved return false, 0, 0;

    current := Windows.GetWindowLongW(hwnd, Windows.GWL_STYLE);
    fullscreen := (cast,force(u32) current & Windows.WS_POPUP) != 0;

    if fullscreen != desire_fullscreen {
        rect: Windows.RECT;
        style: s32;
        ex_style: s32;

        if desire_fullscreen {
            if Windows.GetWindowRect(hwnd, *saved.rect) == 0 return false, 0, 0;

            saved.style = current;
            saved.ex_style = Windows.GetWindowLongW(hwnd, Windows.GWL_EXSTYLE);
            monitor := Windows.MonitorFromWindow(hwnd, Windows.MONITOR_DEFAULTTONEAREST);
            info: Windows.MONITORINFO;
            info.cbSize = size_of(Windows.MONITORINFO);
            if !monitor || Windows.GetMonitorInfoW(monitor, *info) == 0 return false, 0, 0;

            rect = info.rcMonitor;
            style = cast,force(s32) FULLSCREEN_STYLE;
        } else {
            rect = saved.rect;
            style = saved.style;
            ex_style = saved.ex_style;
        }

        win_set_last_error(0);
        old := Windows.SetWindowLongW(hwnd, Windows.GWL_STYLE, style);
        if old == 0 && win_get_last_error() != 0 return false, 0, 0;

        win_set_last_error(0);
        old = Windows.SetWindowLongW(hwnd, Windows.GWL_EXSTYLE, ex_style);
        if old == 0 && win_get_last_error() != 0 return false, 0, 0;
        if Windows.SetWindowPos(hwnd, null, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, Windows.SWP_NOZORDER | Windows.SWP_NOACTIVATE | Windows.SWP_FRAMECHANGED) == 0 return false, 0, 0;
    }

    _, _, w, h, ok := get_dimensions(hwnd, false);

    return ok, w, h;
}

#scope_module
kernel32 :: #library,system "kernel32";
win_set_last_error :: (value: u32) #foreign kernel32 "SetLastError";
win_get_last_error :: () -> u32 #foreign kernel32 "GetLastError";
user32 :: #library,system "user32";
win_get_class_info :: (instance: Windows.HINSTANCE, name: *u16, info: *Windows.WNDCLASSEXW) -> s32 #foreign user32 "GetClassInfoExW";

Basic :: #import "Basic";
Input :: #import "Input";

window_class_registered := false;
window_class_name := u16.[67, 108, 101, 97, 110, 114, 111, 111, 109, 87, 105, 110, 100, 111, 119, 0];

window_utf16 :: (text: string) -> *u16 {
    if text.count > 0x7fffffff return null;
    for byte: text if byte == 0 return null;

    count := Windows.MultiByteToWideChar(65001, 8, text.data, cast(s32) text.count, null, 0);
    if count == 0 && text.count != 0 return null;

    result := cast(*u16) Basic.alloc((cast(int) count + 1) * 2);

    Basic.assert(result != null, "window title allocation failed");

    if count > 0 && Windows.MultiByteToWideChar(65001, 8, text.data, cast(s32) text.count, result, count) != count {
        Basic.free(result);
        return null;
    }

    result[count] = 0;

    return result;
}
