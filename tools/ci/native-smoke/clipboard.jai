// Linked, not run: reading the clipboard needs an X display.
#import "Clipboard";

main :: () {
    text := os_clipboard_get_text();
    if text.count > 1000000 then return;
}
