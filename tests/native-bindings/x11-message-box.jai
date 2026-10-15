// Requires a real Linux X server. The child is an independent native client
// that sends real button events; no message-box implementation is mocked.
X :: #import "X11";
P :: #import "POSIX";

libc :: #library,system "c";
abort :: () #foreign libc;
// Private test constant from pinned X.Org xorgproto-2024.1 include/X11/X.h;
// x11_protocol_probe.c actively asserts this value against the complete header.
native_viewable :: 2;

check :: (value: bool) {
    if !value abort();
}

click_dialog :: (index: s32) {
    display := X.XOpenDisplay(null);

    check(display != null);
    defer X.XCloseDisplay(display);
    root := X.XDefaultRootWindow(display);

    for 0..199 {
        returned_root, parent: X.Window;
        children: *X.Window;
        count: u32;

        check(X.XQueryTree(display, root, *returned_root, *parent, *children, *count) != 0);
        defer if children X.XFree(children);

        for i: 0..cast(s64) count - 1 {
            name: *u8;
            if X.XFetchName(display, children[i], *name) == 0 || !name continue;

            defer X.XFree(name);
            length: s64;
            while name[length] != 0 length += 1;

            title: string = .{ data = name, count = length };
            if title != "Bindings modal event test" continue;

            attributes: X.XWindowAttributes;

            check(X.XGetWindowAttributes(display, children[i], *attributes) != 0);
            if attributes.map_state != native_viewable continue;

            event: X.XEvent;
            event.xbutton.display = display;
            event.xbutton.window = children[i];
            event.xbutton.root = root;
            event.xbutton.x = 32 + 152 * index;
            event.xbutton.y = 208;
            event.xbutton.button = 1;
            event.xbutton.same_screen = 1;
            event.type = X.ButtonPress;
            check(X.XSendEvent(display, children[i], 0, X.ButtonPressMask, *event) != 0);
            event.type = X.ButtonRelease;
            check(X.XSendEvent(display, children[i], 0, X.ButtonReleaseMask, *event) != 0);
            X.XSync(display, 0);

            return;
        }

        check(P.usleep(25000) == 0);
    }

    // A missing or unresponsive dialog is a genuine failed regression.
    abort();
}

main :: () {
    display := X.XOpenDisplay(null);

    check(display != null);
    defer X.XCloseDisplay(display);
    X.x_global_display = display;
    defer X.x_global_display = null;
    other := X.x_toolkit_create_simple_window(display, 0, 32, 32, "Other event owner");

    check(other != 0);
    defer X.XDestroyWindow(display, other);

    styles: [7] X.X_Message_Box_Style = .[
        .ST_ABORT_RETRY_IGNORE, .ST_CANCEL_TRY_CONTINUE, .ST_OK, .ST_OK_CANCEL,
        .ST_RETRY_CANCEL, .ST_YES_NO, .ST_YES_NO_CANCEL
    ];
    buttons: [7] s32 = .[2, 1, 0, 1, 0, 1, 2];
    expected: [7] X.X_Message_Box_Response = .[.IGNORE, .TRY_AGAIN, .OK, .CANCEL, .RETRY, .NO, .CANCEL];

    for i: 0..styles.count - 1 {
        unrelated: X.XEvent;
        unrelated.xclient.type = X.ClientMessage;
        unrelated.xclient.window = other;
        unrelated.xclient.format = 32;
        unrelated.xclient.message_type = X.XInternAtom(display, "BINDING_TEST_OTHER_OWNER", 0);
        unrelated.xclient.data.l[0] = i;
        check(X.XSendEvent(display, other, 0, 0, *unrelated) != 0);
        X.XSync(display, 0);
        child := P.fork();

        check(child >= 0);

        if child == 0 {
            click_dialog(buttons[i]);
            P._exit(0);
        }

        // A failed event client must fail this test instead of hanging CI.
        P.alarm(10);
        response := X.x_message_box(other, "Real Xlib message\nChoose the requested button", "Bindings modal event test", styles[i]);

        P.alarm(0);
        check(response == expected[i]);
        status: s32;

        check(P.waitpid(child, *status, 0) == child && status == 0);
        retained: X.XEvent;

        check(X.XCheckTypedWindowEvent(display, other, X.ClientMessage, *retained) != 0);
        check(retained.xclient.data.l[0] == i);
    }
}
