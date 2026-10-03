// Xlib requests retain the connection's existing asynchronous error handler.
// Integer return values of queued drawing requests are not server statuses.
init_global_display :: () {
    if x_global_display return;
    if XInitThreads() == 0 binding_abort();

    display := XOpenDisplay(null);
    if !display binding_abort();

    x_global_display = display;
    x_global_wm_delete_window = XInternAtom(display, "WM_DELETE_WINDOW", 0);
    x_global_wm_protocols = XInternAtom(display, "WM_PROTOCOLS", 0);
    x_global_net_wm_demands_attention = XInternAtom(display, "_NET_WM_STATE_DEMANDS_ATTENTION", 0);
    x_global_net_wm_state = XInternAtom(display, "_NET_WM_STATE", 0);
    x_global_net_wm_state_fullscreen = XInternAtom(display, "_NET_WM_STATE_FULLSCREEN", 0);
    x_global_xa_clipboard = XInternAtom(display, "CLIPBOARD", 0);
    x_global_xa_utf8 = XInternAtom(display, "UTF8_STRING", 0);
    x_global_xa_targets = XInternAtom(display, "TARGETS", 0);
    x_global_xa_multiple = XInternAtom(display, "MULTIPLE", 0);
    x_global_xa_atom = XInternAtom(display, "ATOM", 0);
    x_global_non_protocol_atom = XInternAtom(display, "JXSelectionWindowProperty", 0);
    x_global_image_bmp = XInternAtom(display, "image/bmp", 0);
    XdndAware = XInternAtom(display, "XdndAware", 0);
    XdndEnter = XInternAtom(display, "XdndEnter", 0);
    XdndPosition = XInternAtom(display, "XdndPosition", 0);
    XdndStatus = XInternAtom(display, "XdndStatus", 0);
    XdndTypeList = XInternAtom(display, "XdndTypeList", 0);
    XdndActionCopy = XInternAtom(display, "XdndActionCopy", 0);
    XdndDrop = XInternAtom(display, "XdndDrop", 0);
    XdndLeave = XInternAtom(display, "XdndLeave", 0);
    XdndFinished = XInternAtom(display, "XdndFinished", 0);
    XdndSelection = XInternAtom(display, "XdndSelection", 0);
    XdndProxy = XInternAtom(display, "XdndProxy", 0);
    XSetLocaleModifiers("");
    x_global_input_method = XOpenIM(display, null, null, null);
    if x_global_input_method {
        x_global_input_context = XCreateIC(x_global_input_method, XNInputStyle,
                                          cast(u64) (XIMPreeditNothing | XIMStatusNothing), cast(*void) null);
    }

    blank: u8;
    pixmap := XCreateBitmapFromData(display, XDefaultRootWindow(display), *blank, 1, 1);

    if pixmap == 0 binding_abort();

    color: XColor;

    x_global_invisible_cursor = XCreatePixmapCursor(display, pixmap, pixmap, *color, *color, 0, 0);
    XFreePixmap(display, pixmap);
    if x_global_invisible_cursor == 0 binding_abort();

    xdnd_context_reset(*x_global_xdnd_context);
}

enable_drag_and_drop :: (window: Window, typelist: [] string) {
    display := x_global_display;
    if !display || window == 0 || typelist.count > 0x7fff_ffff binding_abort();

    atoms: [..] Atom;

    for name: typelist {
        text := Native.terminated_copy(name);
        atom := XInternAtom(display, text, 0);

        Native.release(text);
        if atom == 0 binding_abort();

        B.array_add(*atoms, atom);
    }

    XLockDisplay(display);
    defer XUnlockDisplay(display);
    XChangeProperty(display, window, XdndTypeList, XA_ATOM, 32, PropModeReplace,
                    cast(*u8) atoms.data, cast(s32) atoms.count);

    if drop_info.user_typelist.data {
        previous_allocator := context.allocator;
        context.allocator = binding_typelist_allocator;
        defer context.allocator = previous_allocator;
        B.array_free(drop_info.user_typelist);
    }

    binding_typelist_allocator = context.allocator;
    drop_info.user_typelist = atoms;
    version: u64 = XDND_VERSION;
    current := window;

    while current != 0 {
        XChangeProperty(display, current, XdndAware, XA_ATOM, 32, PropModeReplace, cast(*u8) *version, 1);
        root, parent: Window;
        children: *Window;
        count: u32;

        if XQueryTree(display, current, *root, *parent, *children, *count) == 0 binding_abort();
        if children XFree(children);
        if current == root break;

        current = parent;
    }
}

x_selection_notify_predicate :: (#discard display: *Display, event: *XEvent, arg: XPointer) -> Bool #c_call {
    return ifx event.type == SelectionNotify && event.xselection.requestor == <<cast(*Window) arg then 1 else 0;
}

x_release_clipboard_data :: () {
    display := x_global_display;
    if !display binding_abort();

    XLockDisplay(display);
    defer XUnlockDisplay(display);
    ClipboardOwned.release(*x_global_clipboard_buffer);
}

x_toolkit_create_simple_window :: (dpy: *Display, parent: Window, width: s32, height: s32, title: string) -> Window {
    if !dpy || width <= 0 || height <= 0 return 0;
    if title.count > 0x7fff_ffff binding_abort();

    actual_parent := parent;
    if actual_parent == 0 actual_parent = XDefaultRootWindow(dpy);

    parent_attributes: XWindowAttributes;
    if XGetWindowAttributes(dpy, actual_parent, *parent_attributes) == 0 return 0;

    screen := XScreenNumberOfScreen(parent_attributes.screen);
    window := XCreateSimpleWindow(dpy, actual_parent, 0, 0, cast(u32) width, cast(u32) height,
                                  0, XBlackPixel(dpy, screen), XWhitePixel(dpy, screen));

    if window == 0 return 0;

    terminated := Native.terminated_copy(title);

    defer Native.release(terminated);
    XStoreName(dpy, window, terminated);

    // Modern window managers consume the explicit-length UTF-8 title property.
    name := XInternAtom(dpy, "_NET_WM_NAME", 0);
    utf8 := XInternAtom(dpy, "UTF8_STRING", 0);

    if name == 0 || utf8 == 0 binding_abort();

    XChangeProperty(dpy, window, name, utf8, 8, PropModeReplace, title.data, cast(s32) title.count);

    return window;
}

x_demand_attention :: (display: *Display, win: Window) {
    if !display || win == 0 binding_abort();

    attributes: XWindowAttributes;
    if XGetWindowAttributes(display, win, *attributes) == 0 binding_abort();

    state := XInternAtom(display, "_NET_WM_STATE", 0);
    attention := XInternAtom(display, "_NET_WM_STATE_DEMANDS_ATTENTION", 0);

    if state == 0 || attention == 0 binding_abort();

    event: XEvent;
    event.xclient.type = ClientMessage;
    event.xclient.display = display;
    event.xclient.window = win;
    event.xclient.message_type = state;
    event.xclient.format = 32;
    event.xclient.data.l[0] = _NET_WM_STATE_ADD;
    event.xclient.data.l[1] = cast(s64) attention;
    event.xclient.data.l[3] = 1; // EWMH: source is an ordinary application.
    if XSendEvent(display, attributes.root, 0,
                  SubstructureNotifyMask | SubstructureRedirectMask, *event) == 0 binding_abort();
}

x_message_box :: (parent: Window, msg: string, caption: string, style: X_Message_Box_Style) -> X_Message_Box_Response {
    display := x_global_display;
    owns_display := !display;

    if owns_display display = XOpenDisplay(null);
    if !display binding_abort();

    defer if owns_display XCloseDisplay(display);

    choices: Binding_Message_Choices;

    if style == .ST_ABORT_RETRY_IGNORE {
        choices.labels = .["Abort", "Retry", "Ignore"];
        choices.responses = .[.ABORT, .RETRY, .IGNORE];
        choices.count = 3;
        choices.dismiss = .ABORT;
    } else if style == .ST_CANCEL_TRY_CONTINUE {
        choices.labels = .["Cancel", "Try again", "Continue"];
        choices.responses = .[.CANCEL, .TRY_AGAIN, .CONTINUE];
        choices.count = 3;
    } else if style == .ST_OK {
        choices.labels[0] = "OK";
        choices.responses[0] = .OK;
        choices.count = 1;
        choices.dismiss = .OK;
    } else if style == .ST_OK_CANCEL {
        choices.labels = .["OK", "Cancel", ""];
        choices.responses = .[.OK, .CANCEL, .CANCEL];
        choices.count = 2;
    } else if style == .ST_RETRY_CANCEL {
        choices.labels = .["Retry", "Cancel", ""];
        choices.responses = .[.RETRY, .CANCEL, .CANCEL];
        choices.count = 2;
    } else if style == .ST_YES_NO || style == .ST_YES_NO_CANCEL {
        choices.labels = .["Yes", "No", "Cancel"];
        choices.responses = .[.YES, .NO, .CANCEL];
        choices.count = ifx style == .ST_YES_NO then 2 else 3;
        choices.dismiss = ifx style == .ST_YES_NO then .NO else .CANCEL;
    } else {
        binding_abort();
    }

    window := x_toolkit_create_simple_window(display, 0, 480, 240, caption);
    if window == 0 binding_abort();

    window_live := true;

    defer if window_live XDestroyWindow(display, window);
    if parent != 0 XSetTransientForHint(display, window, parent);

    x_lock_window_size(display, window, 480, 240);
    closing := XInternAtom(display, "WM_DELETE_WINDOW", 0);
    protocols := XInternAtom(display, "WM_PROTOCOLS", 0);

    if closing == 0 || protocols == 0 binding_abort();
    if XSetWMProtocols(display, window, *closing, 1) == 0 binding_abort();

    XSelectInput(display, window, ExposureMask | KeyPressMask | ButtonPressMask | ButtonReleaseMask | StructureNotifyMask);
    XMapRaised(display, window);
    XFlush(display);
    focused: s32;
    pressed: s32 = -1;

    while window_live {
        event: XEvent;

        // Leave other windows' queued events for their owners. The predicate
        // is a C callback carrying only the address of this native window ID.
        XIfEvent(display, *event, binding_message_event, cast(XPointer) *window);

        if event.type == DestroyNotify {
            window_live = false;
            break;
        }

        if event.type == ClientMessage && event.xclient.message_type == protocols &&
           event.xclient.format == 32 && event.xclient.data.l[0] == cast(s64) closing {
            return choices.dismiss;
        }

        if event.type == KeyPress {
            key := XLookupKeysym(*event.xkey, 0);
            if key == XK_Escape return choices.dismiss;
            if key == XK_Return || key == XK_space return choices.responses[focused];
            if key == XK_Tab || key == XK_Right focused = (focused + 1) % choices.count;
            else if key == XK_Left focused = (focused + choices.count - 1) % choices.count;
        } else if event.type == ButtonPress && event.xbutton.button == 1 {
            pressed = binding_message_button(event.xbutton.x, event.xbutton.y, choices.count);
            if pressed >= 0 focused = pressed;
        } else if event.type == ButtonRelease && event.xbutton.button == 1 {
            selected := binding_message_button(event.xbutton.x, event.xbutton.y, choices.count);
            if pressed >= 0 && selected == pressed return choices.responses[selected];
            pressed = -1;
        }

        x_draw_textpanel(display, window, 16, 16, 448, 160, msg);

        for i: 0..choices.count - 1 {
            x_draw_button(display, window, 16 + 152 * i, 192, 136, 32,
                          i == pressed || i == focused, choices.labels[i]);
        }

        XFlush(display);
    }

    return choices.dismiss;
}

x_draw_textpanel :: (dpy: *Display, w: Window, x: s32, y: s32, width: s32, height: s32, msg: string) {
    if width <= 0 || height <= 0 return;

    binding_check_panel(x, y, cast(u32) width, cast(u32) height, msg);
    gc, font, black, white := binding_panel_gc(dpy, w);

    defer XFreeGC(dpy, gc);
    defer XFreeFontInfo(null, font, 1);
    binding_clip_panel(dpy, gc, x, y, cast(u32) width, cast(u32) height);
    XSetForeground(dpy, gc, white);
    XFillRectangle(dpy, w, gc, x, y, cast(u32) width, cast(u32) height);
    XSetForeground(dpy, gc, black);

    line_height := font.ascent + font.descent;
    if line_height <= 0 binding_abort();

    baseline := y + font.ascent + 2;
    begin: s64;

    while begin < msg.count && baseline + font.descent <= y + height {
        end := begin;
        line_width: s64;

        while end < msg.count && msg[end] != #char "\n" {
            measured := line_width + XTextWidth(font, msg.data + end, 1);
            if measured > width - 4 && end > begin break;

            end += 1;
            line_width = measured;
            if measured > width - 4 break;
        }

        if end > begin XDrawString(dpy, w, gc, x + 2, baseline, msg.data + begin, cast(s32) (end - begin));
        if end < msg.count && msg[end] == #char "\n" end += 1;

        begin = end;
        baseline += line_height;
    }
}

x_draw_button :: (dpy: *Display, w: Window, x: s32, y: s32, width: u32, height: u32, pressed: bool, msg: string) {
    if width == 0 || height == 0 return;

    binding_check_panel(x, y, width, height, msg);
    gc, font, black, white := binding_panel_gc(dpy, w);

    defer XFreeGC(dpy, gc);
    defer XFreeFontInfo(null, font, 1);
    binding_clip_panel(dpy, gc, x, y, width, height);
    background := ifx pressed then black else white;
    foreground := ifx pressed then white else black;

    XSetForeground(dpy, gc, background);
    XFillRectangle(dpy, w, gc, x, y, width, height);
    XSetForeground(dpy, gc, foreground);
    XDrawRectangle(dpy, w, gc, x, y, width - 1, height - 1);
    text_width := XTextWidth(font, msg.data, cast(s32) msg.count);
    line_height := font.ascent + font.descent;
    left := cast(s32) (cast(s64) x + (cast(s64) width - cast(s64) text_width) / 2);
    baseline := y + (cast(s32) height - line_height) / 2 + font.ascent;

    XDrawString(dpy, w, gc, left, baseline, msg.data, cast(s32) msg.count);
}

x_draw_string :: (dpy: *Display, d: Drawable, x: s32, y: s32, msg: string) {
    if msg.count > 0x7fff_ffff binding_abort();
    gc := XDefaultGC(dpy, XDefaultScreen(dpy));
    XDrawString(dpy, d, gc, x, y, msg.data, cast(s32) msg.count);
}

x_color_to_pixel :: (dpy: *Display, cmap: Colormap, col: XColor) -> u64 {
    if XAllocColor(dpy, cmap, *col) == 0 binding_abort();
    return col.pixel;
}

x_lock_window_size :: (dpy: *Display, w: Window, width: s32, height: s32) {
    hints: XSizeHints;
    hints.flags = PMinSize | PMaxSize;
    hints.min_width = width;
    hints.max_width = width;
    hints.min_height = height;
    hints.max_height = height;
    XSetWMNormalHints(dpy, w, *hints);
}

x_window_is_ours :: (display: *Display, win: Window) -> bool {
    if display != x_global_display return false;
    for window: x_global_windows if window == win return true;
    return false;
}

xdnd_context_reset :: (dnd: *Xdnd_Context) {
    // Reset the logical operation while retaining the context-owned array's
    // allocation. Its owner remains responsible for final destruction.
    types := dnd.dragger_typelist;
    initial: Xdnd_Context;
    initial.dragger_typelist = types;
    initial.dragger_typelist.count = 0;
    <<dnd = initial;
}

#scope_file
B :: #import "Basic";

binding_typelist_allocator: Allocator;

ClipboardOwned :: #import "internal/bindings/clipboard-owned-data";
Native :: #import "internal/bindings/native";

libc :: #library,system "c";
binding_abort :: () #foreign libc "abort";

Binding_Message_Choices :: struct {
    labels: [3] string;
    responses: [3] X_Message_Box_Response;
    count: s32;
    dismiss: X_Message_Box_Response = .CANCEL;
}

binding_message_event :: (#discard display: *Display, event: *XEvent, arg: XPointer) -> Bool #c_call {
    return ifx event.xany.window == <<cast(*Window) arg then 1 else 0;
}

binding_message_button :: (x: s32, y: s32, count: s32) -> s32 {
    if y < 192 || y >= 224 || x < 16 return -1;
    slot := (x - 16) / 152;
    if slot >= count || (x - 16) % 152 >= 136 return -1;
    return slot;
}

binding_check_panel :: (x: s32, y: s32, width: u32, height: u32, msg: string) {
    // X11's clip rectangle wire coordinates are signed/unsigned 16-bit.
    if x < -32768 || x > 32767 || y < -32768 || y > 32767 || width > 65535 || height > 65535 || msg.count > 0x7fff_ffff {
        binding_abort();
    }
}

binding_clip_panel :: (dpy: *Display, gc: GC, x: s32, y: s32, width: u32, height: u32) {
    rectangle: XRectangle;
    rectangle.x = cast(s16) x;
    rectangle.y = cast(s16) y;
    rectangle.width = cast(u16) width;
    rectangle.height = cast(u16) height;
    XSetClipRectangles(dpy, gc, 0, 0, *rectangle, 1, 0);
}

binding_panel_gc :: (dpy: *Display, window: Window) -> GC, *XFontStruct, u64, u64 {
    if !dpy || window == 0 binding_abort();

    attributes: XWindowAttributes;
    if XGetWindowAttributes(dpy, window, *attributes) == 0 binding_abort();

    gc := XCreateGC(dpy, window, 0, null);
    if !gc binding_abort();

    font := XQueryFont(dpy, XGContextFromGC(gc));
    if !font binding_abort();

    screen := XScreenNumberOfScreen(attributes.screen);

    return gc, font, XBlackPixel(dpy, screen), XWhitePixel(dpy, screen);
}
