#import "Window_Creation";
#import "Clipboard";
#import "Basic";

main :: () {}

// Source only: validate the ordinary public Window producer with Clipboard,
// including the private shared connection initialization. Never run on Mac.
exercise_window :: () {
    win := create_window(32, 32, "clipboard source contract");

    os_clipboard_set_text(win, "café");
    text := os_clipboard_get_text(win);

    free(text);
    os_clipboard_set_bitmap(win, 1, 1, null, 3);
}
