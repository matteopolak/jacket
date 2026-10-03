os_clipboard_get_text :: (win: Window) -> string {
    bytes: *u8;
    count: u64;
    result := native_read(win, *bytes, *count);

    defer if bytes native_free(bytes);
    if !clipboard_check(result == 0, "X11 clipboard text retrieval failed") return alloc_string(0);
    if !clipboard_check(count <= 0x7fffffffffffffff, "X11 clipboard text exceeds string storage limits") return alloc_string(0);

    return copy_string(.{ cast(int) count, bytes });
}

os_clipboard_get_text :: () -> string {
    return os_clipboard_get_text(cast(Window) 0);
}

os_clipboard_set_text :: (win: Window, text: string) {
    result := native_text(win, text.data, cast(u64) text.count);
    clipboard_check(result == 0, "X11 clipboard text write failed");
}

os_clipboard_set_text :: (text: string) {
    os_clipboard_set_text(cast(Window) 0, text);
}

os_clipboard_set_bitmap :: (win: Window, width: s32, height: s32, pixels: *u8, channels: s32, flip := false) {
    result := native_bitmap(win, width, height, pixels, channels, cast(s32) flip);
    clipboard_check(result == 0, "X11 clipboard bitmap write failed");
}

os_clipboard_set_bitmap :: (width: s32, height: s32, pixels: *u8, channels: s32, flip := false) {
    os_clipboard_set_bitmap(cast(Window) 0, width, height, pixels, channels, flip);
}

#scope_module
// Window is the canonical public X11 XID type, never a replacement record.
#import "X11";

backend :: #library,system "portable_clipboard";
// Native archive references are not Jai foreign roots. Retain its real SDK
// dependencies even when the canonical X11 import is used only for XID types.
#library,system,link_always "X11";
thread_backend :: #library,system,link_always "pthread";
native_read :: (parent: Window, text: **u8, size: *u64) -> s32 #foreign backend "portable_clipboard_read";
native_text :: (parent: Window, text: *u8, size: u64) -> s32 #foreign backend "portable_clipboard_text";
native_bitmap :: (parent: Window, width: s32, height: s32, pixels: *u8, channels: s32, flip: s32) -> s32 #foreign backend "portable_clipboard_bitmap";
native_free :: (text: *void) #foreign backend "portable_clipboard_free";

clipboard_check :: (success: bool, message: string) -> bool {
    if success return true;
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
    return false;
}
