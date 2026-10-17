#import "Clipboard";
#import "Basic";
X :: #import "X11";

// Source-only: taking procedure addresses checks all reviewed Linux overloads
// without connecting to a display or changing the session clipboard.
main :: () {
    get: #type () -> string = os_clipboard_get_text;
    get_window: #type (win: X.Window) -> string = os_clipboard_get_text;
    text: #type (text: string) = os_clipboard_set_text;
    text_window: #type (win: X.Window, text: string) = os_clipboard_set_text;
    bitmap: #type (width: s32, height: s32, pixels: *u8, channels: s32, flip: bool) = os_clipboard_set_bitmap;
    bitmap_window: #type (win: X.Window, width: s32, height: s32, pixels: *u8, channels: s32, flip: bool) = os_clipboard_set_bitmap;

    assert(get != null && get_window != null && text != null && text_window != null && bitmap != null && bitmap_window != null);
}

exercise_defaults :: (win: X.Window) {
    os_clipboard_set_bitmap(width = 1, height = 1, pixels = null, channels = 3);
    os_clipboard_set_bitmap(win = win, width = 1, height = 1, pixels = null, channels = 4);
}
