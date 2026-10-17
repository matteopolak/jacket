// Real X server executable; Linux cross-target checks only validate source.
X :: #import "X11";

libc :: #library,system "c";
abort :: () #foreign libc;
native_x11 :: #library,system "X11";
pixel :: (image: *X.XImage, x: s32, y: s32) -> u64 #foreign native_x11 "XGetPixel";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    display := X.XOpenDisplay(null);

    check(display != null);
    defer X.XCloseDisplay(display);
    window := X.x_toolkit_create_simple_window(display, 0, 96, 64, "binding window");

    check(window != 0);
    defer X.XDestroyWindow(display, window);
    X.XSelectInput(display, window, X.StructureNotifyMask);
    X.XMapWindow(display, window);

    while true {
        event: X.XEvent;
        X.XWindowEvent(display, window, X.StructureNotifyMask, *event);
        if event.type == X.MapNotify break;
    }

    name: *u8;

    check(X.XFetchName(display, window, *name) != 0 && name != null);
    defer X.XFree(name);
    text: string = .{ data = name, count = 14 };

    check(text == "binding window" && name[14] == 0);
    X.x_demand_attention(display, window);
    X.x_draw_textpanel(display, window, 0, 0, 48, 24, "Panel\ntext");
    X.x_draw_button(display, window, 48, 0, 48, 24, true, "On");
    X.x_draw_button(display, window, 0, 32, 48, 24, false, "Off");
    // Flush queued requests and expose real server errors to the normal handler.
    X.XSync(display, 0);
    image := X.XGetImage(display, window, 0, 0, 96, 64, X.XAllPlanes(), X.ZPixmap);

    check(image != null);
    defer X.XDestroyImage(image);
    screen := X.XDefaultScreen(display);

    check(pixel(image, 49, 1) == X.XBlackPixel(display, screen));
    check(pixel(image, 2, 34) == X.XWhitePixel(display, screen));
    check(X.x_toolkit_create_simple_window(display, 0, 0, 64, "invalid") == 0);
}
