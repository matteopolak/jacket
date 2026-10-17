// Reviewed x11.md helper contracts; requires a real Linux X server to execute.
X :: #import "X11";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    requestor: X.Window = 731;
    event: X.XEvent;
    event.xselection.type = X.SelectionNotify;
    event.xselection.requestor = requestor;
    check(X.x_selection_notify_predicate(null, *event, cast(X.XPointer) *requestor) == 1);
    event.xselection.target = 987;
    check(X.x_selection_notify_predicate(null, *event, cast(X.XPointer) *requestor) == 1);
    event.xselection.requestor += 1;
    check(X.x_selection_notify_predicate(null, *event, cast(X.XPointer) *requestor) == 0);
    event.xselection.requestor = requestor;
    event.type = X.SelectionRequest;
    check(X.x_selection_notify_predicate(null, *event, cast(X.XPointer) *requestor) == 0);

    X.init_global_display();
    display := X.x_global_display;

    check(display != null);
    X.init_global_display();
    check(X.x_global_display == display);
    check(X.x_global_non_protocol_atom == X.XInternAtom(display, "JXSelectionWindowProperty", 1));
    check(X.x_global_image_png == 0 && X.x_global_image_jpeg == 0);
    parent := X.XCreateSimpleWindow(display, X.XDefaultRootWindow(display), 0, 0, 32, 32, 0, 0, 0);
    child := X.XCreateSimpleWindow(display, parent, 0, 0, 16, 16, 0, 0, 0);

    defer X.XDestroyWindow(display, child);
    defer X.XDestroyWindow(display, parent);
    types := string.["text/plain", "text/uri-list"];

    X.enable_drag_and_drop(child, types);
    check(X.drop_info.user_typelist.count == 2);

    kind: X.Atom;
    bits: s32;
    count, remaining: u64;
    data: *u8;

    check(X.XGetWindowProperty(display, child, X.XdndTypeList, 0, 2, 0, X.XA_ATOM,
                              *kind, *bits, *count, *remaining, *data) == 0);
    check(kind == X.XA_ATOM && bits == 32 && count == 2 && remaining == 0);
    native_types := cast(*X.Atom) data;

    check(native_types[0] == X.XInternAtom(display, "text/plain", 1));
    check(native_types[1] == X.XInternAtom(display, "text/uri-list", 1));
    X.XFree(data);

    for window: X.Window.[child, parent, X.XDefaultRootWindow(display)] {
        check(X.XGetWindowProperty(display, window, X.XdndAware, 0, 1, 0, X.XA_ATOM,
                                  *kind, *bits, *count, *remaining, *data) == 0);
        check(kind == X.XA_ATOM && bits == 32 && count == 1);
        check(<<cast(*u64) data == cast(u64) X.XDND_VERSION);
        X.XFree(data);
    }

    X.enable_drag_and_drop(child, string.[]);
    check(X.drop_info.user_typelist.count == 0);
    check(X.XGetWindowProperty(display, child, X.XdndTypeList, 0, 2, 0, X.XA_ATOM,
                              *kind, *bits, *count, *remaining, *data) == 0);
    check(kind == X.XA_ATOM && bits == 32 && count == 0 && remaining == 0);
    if data X.XFree(data);

    X.XSync(display, 0);
}
