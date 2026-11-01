// test: only(.WINDOWS, reason = "Win32 WM_DPICHANGED suggested-rectangle handling")
#import "Basic";
W :: #import "Window_Creation";
N :: #import "Windows";
I :: #import "Input";

main :: () {
    window := W.create_window(320, 200, "DPI rectangle test");

    assert(window != null);
    defer N.DestroyWindow(window);
    assert(W.get_pixel_scale(window) >= 1);
    I.update_window_events();

    suggested := N.RECT.{ 100, 120, 612, 520 };

    assert(I.MyWindowProc(window, 0x02e0, 192 | (192 << 16), cast(s64) *suggested) == 0);
    actual: N.RECT;

    assert(N.GetWindowRect(window, *actual) != 0);
    assert(actual.left == suggested.left && actual.top == suggested.top);
    assert(actual.right == suggested.right && actual.bottom == suggested.bottom);
    records := I.get_window_resizes();
    found := false;

    for records if it.window == window && it.width > 0 && it.height > 0 found = true;

    assert(found, "applying the DPI rectangle must produce the existing resize record");
    print("ok\n");
}
