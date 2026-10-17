#import "Basic";
#import "Clipboard";

main :: () {
    get: #type () -> string = os_clipboard_get_text;
    text: #type (text: string) = os_clipboard_set_text;
    bitmap: #type (width: s32, height: s32, pixels: *u8, channels: s32, flip: bool) = os_clipboard_set_bitmap;

    assert(get != null && text != null && bitmap != null);

    #if OS != .LINUX {
        empty: #type () = os_clipboard_empty;
        assert(empty != null);
    }
}
