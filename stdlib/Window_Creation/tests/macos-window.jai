#import "Basic";
WC :: #import "Window_Creation";
WT :: #import "Window_Type";
#load "../../internal/window/cocoa.jai";
AppKit :: #import "Objective_C/AppKit";

main :: () {
    WC.init_mac_app();
    assert(WC.application_initialized);
    window: WT.Window_Type = create_content_window(320, 200, "Clean-room native window test", 40, 200);

    assert(window != WT.INVALID_WINDOW);
    defer close_window(window);
    #assert type_of(window) == *AppKit.NSWindow;
    x, y, width, height, ok := WC.get_dimensions(window, true);

    assert(ok && x == 40 && y == 200 && width == 320 && height == 200);
    _, _, left_width, left_height, left_ok := WC.get_dimensions(window, false);

    assert(left_ok && left_width == width && left_height == height);
    _, _, invalid_ok := WC.get_mouse_pointer_position(WT.INVALID_WINDOW, false);

    assert(!invalid_ok);
    _, _, invalid_width, invalid_height, valid := WC.get_dimensions(WT.INVALID_WINDOW, true);

    assert(!valid && invalid_width == 0 && invalid_height == 0);
    print("ok\n");
}
