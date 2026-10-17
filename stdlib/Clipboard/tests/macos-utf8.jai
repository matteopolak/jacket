// Reviewed Clipboard API; UTF-8 and embedded NUL use explicit byte lengths.
#import "Basic";
#import "Clipboard";

main :: () {
    #if OS == .MACOS {
        original := os_clipboard_get_text();

        defer free(original);
        defer os_clipboard_set_text(original);

        for text: string.["", "π 🪟 日本", "a\0b"] {
            os_clipboard_set_text(text);
            result := os_clipboard_get_text();
            assert(result == text, "clipboard explicit UTF-8 length round trip failed");
            free(result);
        }
    }
}
