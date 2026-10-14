#import "Basic";
W :: #import "Window_Creation";
I :: #import "Input";

main :: () {
    window := W.create_window(320, 200, "Public native window", 40, 200);

    assert(window != W.WT.INVALID_WINDOW);
    _, _, width, height, ok := W.get_dimensions(window, true);

    assert(ok && width == 320 && height == 200);
    I.update_window_events();

    #if OS == .MACOS {
        O :: #import "Objective_C";
        objc :: #library,system "objc";
        message :: () #foreign objc "objc_msgSend";
        Close :: #type (receiver: *void, selector: O.Selector) #c_call;
        close := cast(Close) message;

        close(window, O.selector("close"));
        O.release(window);
    } else #if OS == .WINDOWS {
        Native :: #import "Windows";
        assert(Native.DestroyWindow(window) != 0);
    } else #if OS == .LINUX {
        Native :: #import "X11";
        Native.XDestroyWindow(Native.x_global_display, window);
        Native.XSync(Native.x_global_display, 0);
        I.update_window_events();
    }

    print("ok\n");
}
