os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response {
    display := X.XOpenDisplay(null);
    if !display {
        log_error("Debug: cannot open X11 assertion dialog\n");
        return .Abort;
    }

    defer X.XCloseDisplay(display);
    screen := X.XDefaultScreen(display);
    window := X.XCreateSimpleWindow(display, X.XRootWindow(display, screen), 0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, 1, X.XBlackPixel(display, screen), X.XWhitePixel(display, screen));

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
            for 0..TEXT_ROWS - 1 {
                start := it * TEXT_ROW_CHARS;
                if start >= text.count break;
                count := min(TEXT_ROW_CHARS, text.count - start);
                X.XDrawString(display, window, gc, TEXT_LEFT, cast(s32) (TEXT_TOP + it * TEXT_ROW_HEIGHT), text.data + start, cast(s32) count);
            }

            labels: [3] string = .["Retry (r)", "Ignore (i)", "Abort (a)"];
            for label, index: labels {
                x := cast(s32) (BUTTON_LEFT + index * BUTTON_STEP);
                X.XDrawRectangle(display, window, gc, x, BUTTON_TOP, BUTTON_WIDTH, BUTTON_HEIGHT);
                X.XDrawString(display, window, gc, x + LABEL_INSET_X, LABEL_BASELINE, label.data, cast(s32) label.count);
            }

            X.XFlush(display);
        }

        if event.type == X.ButtonPress && event.xbutton.button == 1 && event.xbutton.y >= BUTTON_TOP && event.xbutton.y <= BUTTON_TOP + BUTTON_HEIGHT {
            responses :: Assert_Response.[.Debug, .Ignore, .Abort];
            for responses {
                left := BUTTON_LEFT + it_index * BUTTON_STEP;
                if event.xbutton.x >= left && event.xbutton.x <= left + BUTTON_WIDTH return it;
            }
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
// The window: three rows of the message, then the three buttons side by side.
WINDOW_WIDTH :: 720;
WINDOW_HEIGHT :: 200;
TEXT_ROWS :: 3;
TEXT_ROW_CHARS :: 100;
TEXT_LEFT :: 15;
TEXT_TOP :: 30;
TEXT_ROW_HEIGHT :: 25;
BUTTON_LEFT :: 20;
BUTTON_STEP :: 230;
BUTTON_TOP :: 140;
BUTTON_WIDTH :: 210;
BUTTON_HEIGHT :: 40;
LABEL_INSET_X :: 10;
LABEL_BASELINE :: 165;

#import "Basic";
X :: #import "X11";
