os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response {
    display := X.XOpenDisplay(null);
    if !display {
        log_error("Debug: cannot open X11 assertion dialog\n");
        return .Abort;
    }

    defer X.XCloseDisplay(display);
    screen := X.XDefaultScreen(display);
    window := X.XCreateSimpleWindow(display, X.XRootWindow(display, screen), 0, 0, 720, 200, 1, X.XBlackPixel(display, screen), X.XWhitePixel(display, screen));

    if window == 0 return .Abort;

    defer X.XDestroyWindow(display, window);
    gc := X.XCreateGC(display, window, 0, null);
    if !gc return .Abort;

    defer X.XFreeGC(display, gc);
    X.XStoreName(display, window, "Assertion failed");
    close_atom := X.XInternAtom(display, "WM_DELETE_WINDOW", 0);
    protocols := X.XInternAtom(display, "WM_PROTOCOLS", 0);

    if X.XSetWMProtocols(display, window, *close_atom, 1) == 0 return .Abort;

    X.XSelectInput(display, window, X.ExposureMask | X.ButtonPressMask | X.KeyPressMask);
    X.XMapRaised(display, window);
    text := tprint("%:%:%: %", loc.fully_pathed_filename, loc.line_number, loc.character_number, message);

    while true {
        event: X.XEvent;

        X.XNextEvent(display, *event);

        if event.type == X.Expose {
            // Full diagnostics were printed by interactive_assert. The modal
            // choice window uses bounded, legacy Xlib text rows.
            for 0..2 {
                start := it * 100;
                if start >= text.count break;
                count := min(100, text.count - start);
                X.XDrawString(display, window, gc, 15, cast(s32) (30 + it * 25), text.data + start, cast(s32) count);
            }

            labels: [3] string = .["Retry (r)", "Ignore (i)", "Abort (a)"];
            for label, index: labels {
                x := cast(s32) (20 + index * 230);
                X.XDrawRectangle(display, window, gc, x, 140, 210, 40);
                X.XDrawString(display, window, gc, x + 10, 165, label.data, cast(s32) label.count);
            }

            X.XFlush(display);
        }

        if event.type == X.ButtonPress && event.xbutton.button == 1 && event.xbutton.y >= 140 && event.xbutton.y <= 180 {
            x := event.xbutton.x;
            if x >= 20 && x <= 230 return .Debug;
            if x >= 250 && x <= 460 return .Ignore;
            if x >= 480 && x <= 690 return .Abort;
        }

        if event.type == X.KeyPress {
            key := X.XLookupKeysym(*event.xkey, 0);
            if key == X.XK_r || key == X.XK_R return .Debug;
            if key == X.XK_i || key == X.XK_I return .Ignore;
            if key == X.XK_a || key == X.XK_A || key == X.XK_Escape return .Abort;
        }

        if event.type == X.ClientMessage && event.xclient.message_type == protocols && event.xclient.format == 32 && cast(u64) event.xclient.data.l[0] == close_atom return .Abort;
    }

    return .Abort;
}

#scope_file
#import "Basic";
X :: #import "X11";
