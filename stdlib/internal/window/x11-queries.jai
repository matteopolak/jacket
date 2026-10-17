get_dimensions :: (win: Window, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true) {
    if win == None return 0, 0, 0, 0, false;

    dpy := Connection.get_display();
    if !dpy return 0, 0, 0, 0, false;

    attributes: XWindowAttributes;
    if XGetWindowAttributes(dpy, win, *attributes) == 0 return 0, 0, 0, 0, false;

    child: Window;
    root_x, root_y: s32;

    if XTranslateCoordinates(dpy, win, attributes.root, 0, 0, *root_x, *root_y, *child) == 0 return 0, 0, 0, 0, false;
    if right_handed root_y = XHeightOfScreen(attributes.screen) - root_y - attributes.height;

    return root_x, root_y, attributes.width, attributes.height, true;
}

get_mouse_pointer_position :: (dpy: *Display, win: Window, right_handed: bool) -> x: int, y: int, success: bool {
    if !dpy || win == None return 0, 0, false;

    root, child: Window;
    root_x, root_y, window_x, window_y: s32;
    mask: u32;

    if XQueryPointer(dpy, win, *root, *child, *root_x, *root_y, *window_x, *window_y, *mask) == 0 return 0, 0, false;

    if right_handed {
        attributes: XWindowAttributes;
        if XGetWindowAttributes(dpy, win, *attributes) == 0 return 0, 0, false;
        window_y = attributes.height - 1 - window_y;
    }

    return window_x, window_y, true;
}

get_mouse_pointer_position :: (win: Window, right_handed: bool) -> x: int, y: int, success: bool {
    if win == None return 0, 0, false;
    return get_mouse_pointer_position(Connection.get_display(), win, right_handed);
}

get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool {
    dpy := Connection.get_display();
    if !dpy return 0, 0, false;
    return get_mouse_pointer_position(dpy, XDefaultRootWindow(dpy), right_handed);
}

toggle_fullscreen :: (win: Window, desire_fullscreen: bool, saved: *Saved_Window_Info) -> success: bool, width: s32, height: s32 {
    if win == None return false, 0, 0;

    dpy := Connection.get_display();
    if !dpy return false, 0, 0;

    fullscreen := XInternAtom(dpy, "_NET_WM_STATE_FULLSCREEN", 1);
    state := XInternAtom(dpy, "_NET_WM_STATE", 1);

    if fullscreen == None || state == None return false, 0, 0;

    root := XDefaultRootWindow(dpy);

    event: XEvent;
    event.xclient.type = ClientMessage;
    event.xclient.window = win;
    event.xclient.message_type = state;
    event.xclient.format = 32;
    event.xclient.data.l[0] = cast(s64) desire_fullscreen;
    event.xclient.data.l[1] = cast(s64) fullscreen;
    event.xclient.data.l[3] = 1;
    if XSendEvent(dpy, root, 0, SubstructureRedirectMask | SubstructureNotifyMask, *event) == 0 return false, 0, 0;

    XFlush(dpy);
    x, y, width, height, ok := get_dimensions(win, false);

    // EWMH requests are asynchronous; dimensions describe the current native window.
    return ok, width, height;
}

