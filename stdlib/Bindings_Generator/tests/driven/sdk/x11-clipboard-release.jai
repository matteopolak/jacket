// Reviewed module-owned clipboard buffers, public storage and initialized
// Display locking. Native execution requires an actual X server.
#import "Basic";
X :: #import "X11";

main :: () {
    display := X.XOpenDisplay(null);

    assert(display != null);
    X.x_global_display = display;
    X.x_global_clipboard_buffer.text_data = alloc_string(17);
    X.x_global_clipboard_buffer.rgb_data = cast(*u8) alloc(24);
    X.x_global_clipboard_buffer.width = 3;
    X.x_global_clipboard_buffer.height = 2;
    X.x_global_clipboard_buffer.pitch = 12;
    X.x_release_clipboard_data();
    assert(X.x_global_clipboard_buffer.text_data.data == null);
    assert(X.x_global_clipboard_buffer.text_data.count == 0);
    assert(X.x_global_clipboard_buffer.rgb_data == null);
    assert(X.x_global_clipboard_buffer.width == 0 && X.x_global_clipboard_buffer.height == 0 && X.x_global_clipboard_buffer.pitch == 0);
    X.x_release_clipboard_data();
    X.x_global_display = null;
    X.XCloseDisplay(display);
}
