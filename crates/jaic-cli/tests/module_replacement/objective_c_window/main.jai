// Window_Creation on macOS with a project-owned Objective_C module.
#import "Basic";
#import "Window_Creation";
#import "Input";
#import "Clipboard";
#import "GL";
Native :: #import "Objective_C";

main :: () {
    rect: Native.NSRect;
    window := create_window(640, 480, "replacement", 0, 0);

    update_window_events();
    glFlush();
    print("% % %\n", rect.size.width, window, os_clipboard_get_text());
}
