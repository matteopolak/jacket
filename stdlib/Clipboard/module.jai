// Reads and writes the system clipboard: text with [os_clipboard_get_text] and
// [os_clipboard_set_text], images with [os_clipboard_set_bitmap]. On Linux it uses X11, and each
// procedure has an overload that takes the X11 window to act for. `os_clipboard_empty` is
// missing there.
//
// A failure the system reports fails an `assert`.
//
// ```jai,no_run
// os_clipboard_set_text("copied");
// text := os_clipboard_get_text();
// assert(text == "copied");
// free(text);
// ```

#if OS == .MACOS {
    #load "../internal/clipboard/cocoa.jai";
} else #if OS == .WINDOWS {
    #load "../internal/clipboard/win32.jai";
} else #if OS == .LINUX {
    #load "../internal/clipboard/linux.jai";
} else {
    #assert false, "Clipboard requires macOS, Windows or Linux/X11";
}

#scope_module
#import "Basic";
