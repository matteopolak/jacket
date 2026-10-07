// Window_Creation and Input against the real window system (AppKit, Win32 or X11): windows open
// and report their size through the event pump, key and mouse events posted to a window come back
// as Input events routed to that window (a click must not end the program), and closing a window
// the way its close button does reports QUIT for that window only. Linux needs a display (CI runs
// the tests under Xvfb). The wasm targets have no window system.
#import "Basic";

#import "Window_Creation";
#import "Input";

// Letter keys are their upper-case character.
KEY_A :: cast(Key_Code) #char "A";

// What the pump reported for one window.
Seen :: struct {
    width: s32;
    height: s32;
    resized: bool;
    quits: int;
    downs: [..] Key_Code;
    ups: [..] Key_Code;
    text: [..] u32;
}

// Runs the event pump for up to `frames` frames, recording what arrives for each window.
// Stops early once `done` says what the caller waits for has arrived.
pump :: (windows: [] Window_Type, seen: [] Seen, done: (seen: [] Seen) -> bool, frames := 200) -> bool {
    for 1..frames {
        update_window_events();
        for resize: get_window_resizes() {
            for windows if it == resize.window {
                seen[it_index].resized = true;
                seen[it_index].width = resize.width;
                seen[it_index].height = resize.height;
            }
        }
        for routed: routed_events_this_frame {
            for windows if it == routed.window {
                record := *seen[it_index];
                event := routed.event;
                if event.type == .QUIT record.quits += 1;
                if event.type == .KEYBOARD {
                    if event.key_pressed array_add(*record.downs, event.key_code);
                    else array_add(*record.ups, event.key_code);
                }
                if event.type == .TEXT_INPUT array_add(*record.text, event.utf32);
            }
        }
        if done(seen) return true;
        sleep_milliseconds(5);
    }
    return false;
}

has :: (codes: [] Key_Code, wanted: Key_Code) -> bool {
    for codes if it == wanted return true;
    return false;
}

main :: () {
    first := create_window(320, 240, "jaic input test one");
    assert(first != INVALID_WINDOW_HANDLE, "create_window failed");
    second := create_window(200, 150, "jaic input test two");
    assert(second != INVALID_WINDOW_HANDLE && second != first, "second create_window failed");
    windows: [2] Window_Type = .[cast(Window_Type) first, cast(Window_Type) second];
    seen: [2] Seen;

    // Both windows report a size once they are on screen.
    sized := pump(windows, seen, (seen: [] Seen) -> bool {
        return seen[0].resized && seen[1].resized;
    });
    assert(sized, "no resize event for a new window (first %, second %)", seen[0].resized, seen[1].resized);
    assert(seen[0].width > 0 && seen[0].height > 0, "first window size % x %", seen[0].width, seen[0].height);
    _, _, width, height, measured := get_dimensions(first, false);
    assert(measured && width > 0 && height > 0, "get_dimensions % x % (%)", width, height, measured);
    assert(width == seen[0].width && height == seen[0].height, "resize % x % vs get_dimensions % x %", seen[0].width, seen[0].height, width, height);
    _, _, pointer_ok := get_mouse_pointer_position(first, false);
    assert(pointer_ok, "get_mouse_pointer_position failed");

    // A key typed into the second window is routed there, with its text.
    post_key(second, true);
    post_key(second, false);
    typed := pump(windows, seen, (seen: [] Seen) -> bool {
        return has(seen[1].ups, KEY_A);
    });
    assert(typed, "key events for the second window: downs %, ups %", seen[1].downs, seen[1].ups);
    assert(has(seen[1].downs, KEY_A), "no key press for the second window: %", seen[1].downs);
    assert(!has(seen[0].downs, KEY_A), "the first window got the second window's key");
    #if OS != .WINDOWS {
        // Windows turns posted WM_KEYDOWNs into characters only for the focused window.
        found_text := false;
        for seen[1].text if it == #char "a" found_text = true;
        assert(found_text, "no text input for the typed key: %", seen[1].text);
    }

    // A click in the first window is a mouse button event, not a request to quit.
    post_click(first, true);
    post_click(first, false);
    clicked := pump(windows, seen, (seen: [] Seen) -> bool {
        return has(seen[0].ups, .MOUSE_BUTTON_LEFT);
    });
    assert(clicked, "mouse button events for the first window: downs %, ups %", seen[0].downs, seen[0].ups);
    assert(has(seen[0].downs, .MOUSE_BUTTON_LEFT), "no mouse press: %", seen[0].downs);
    pump(windows, seen, (seen: [] Seen) -> bool {
        return false;
    }, 20);
    assert(seen[0].quits == 0 && seen[1].quits == 0, "QUIT without closing (% and %)", seen[0].quits, seen[1].quits);

    // Over Input's windows the arrow comes back as the pointer moves, unless the program
    // handles the cursor itself (set_custom_cursor_handling).
    for custom: bool.[false, true] {
        set_custom_cursor_handling(custom);
        show_hand(first);
        move_pointer(first);
        pump(windows, seen, (seen: [] Seen) -> bool {
            return false;
        }, 10);
        assert(hand_shown(first) == custom, "custom cursor handling %: hand cursor shown %", custom, hand_shown(first));
    }
    set_custom_cursor_handling(false);

    // Closing the second window as its close button would reports QUIT for it alone.
    close_like_the_user(second);
    closed := pump(windows, seen, (seen: [] Seen) -> bool {
        return seen[1].quits > 0;
    });
    assert(closed, "no QUIT after closing the second window");
    assert(seen[0].quits == 0, "closing the second window sent QUIT to the first");
    close_like_the_user(first);
    closed = pump(windows, seen, (seen: [] Seen) -> bool {
        return seen[0].quits > 0;
    });
    assert(closed, "no QUIT after closing the first window");
    print("ok\n");
}

#if OS == .MACOS {
    #import "Objective_C";
    #import "Objective_C/AppKit";

    INVALID_WINDOW_HANDLE :: cast(*NSWindow) null;

    window_number :: (window: *NSWindow) -> s64 {
        call: (receiver: *void, selector: SEL) -> s64 #c_call;
        call = cast(type_of(call)) objc_msgSend;
        return call(window, sel_registerName("windowNumber"));
    }

    utf8_string :: (text: *u8) -> *void {
        call: (receiver: *void, selector: SEL, text: *u8) -> *void #c_call;
        call = cast(type_of(call)) objc_msgSend;
        return call(cast(*void) objc_getClass("NSString"), sel_registerName("stringWithUTF8String:"), text);
    }

    post :: (event: *void) {
        call: (receiver: *void, selector: SEL, event: *void, at_start: BOOL) #c_call;
        call = cast(type_of(call)) objc_msgSend;
        call(NSApplication.sharedApplication(), sel_registerName("postEvent:atStart:"), event, NO);
    }

    post_key :: (window: *NSWindow, down: bool) {
        make: (receiver: *void, selector: SEL, type: u64, location: NSPoint, flags: u64, time: float64, number: s64, ctx: *void, characters: *void, unmodified: *void, repeat: BOOL, code: u16) -> *void #c_call;
        make = cast(type_of(make)) objc_msgSend;
        a := utf8_string("a");
        // NSEventTypeKeyDown, NSEventTypeKeyUp; key code 0 is the A key.
        event := make(cast(*void) objc_getClass("NSEvent"),
            sel_registerName("keyEventWithType:location:modifierFlags:timestamp:windowNumber:context:characters:charactersIgnoringModifiers:isARepeat:keyCode:"),
            ifx down then 10 else 11, .{ 10, 10 }, 0, 0, window_number(window), null, a, a, NO, 0);
        assert(event != null, "NSEvent key event");
        post(event);
    }

    post_click :: (window: *NSWindow, down: bool) {
        make: (receiver: *void, selector: SEL, type: u64, location: NSPoint, flags: u64, time: float64, number: s64, ctx: *void, event_number: s64, clicks: s64, pressure: float32) -> *void #c_call;
        make = cast(type_of(make)) objc_msgSend;
        // NSEventTypeLeftMouseDown, NSEventTypeLeftMouseUp, in the window's content area.
        event := make(cast(*void) objc_getClass("NSEvent"),
            sel_registerName("mouseEventWithType:location:modifierFlags:timestamp:windowNumber:context:eventNumber:clickCount:pressure:"),
            ifx down then 1 else 2, .{ 40, 40 }, 0, 0, window_number(window), null, 0, 1, 1);
        assert(event != null, "NSEvent mouse event");
        post(event);
    }

    close_like_the_user :: (window: *NSWindow) {
        call: (receiver: *void, selector: SEL, sender: *void) #c_call;
        call = cast(type_of(call)) objc_msgSend;
        call(window, sel_registerName("performClose:"), null);
    }

    show_hand :: (window: *NSWindow) {
        NSCursor.set(NSCursor.pointingHandCursor());
    }

    move_pointer :: (window: *NSWindow) {
        make: (receiver: *void, selector: SEL, type: u64, location: NSPoint, flags: u64, time: float64, number: s64, ctx: *void, event_number: s64, clicks: s64, pressure: float32) -> *void #c_call;
        make = cast(type_of(make)) objc_msgSend;
        // NSEventTypeMouseMoved in the window's content area.
        event := make(cast(*void) objc_getClass("NSEvent"),
            sel_registerName("mouseEventWithType:location:modifierFlags:timestamp:windowNumber:context:eventNumber:clickCount:pressure:"),
            5, .{ 50, 50 }, 0, 0, window_number(window), null, 0, 0, 0);
        assert(event != null, "NSEvent mouse moved event");
        post(event);
    }

    hand_shown :: (window: *NSWindow) -> bool {
        call: (receiver: *void, selector: SEL) -> *void #c_call;
        call = cast(type_of(call)) objc_msgSend;
        return call(cast(*void) objc_getClass("NSCursor"), sel_registerName("currentCursor")) == cast(*void) NSCursor.pointingHandCursor();
    }
} else #if OS == .WINDOWS {
    #import "Windows";

    INVALID_WINDOW_HANDLE :: cast(HWND) null;

    user32 :: #system_library "user32";
    PostMessageW :: (window: HWND, message: u32, wparam: u64, lparam: s64) -> s32 #foreign user32;
    SendMessageW :: (window: HWND, message: u32, wparam: u64, lparam: s64) -> s64 #foreign user32;

    post_key :: (window: HWND, down: bool) {
        // WM_KEYDOWN / WM_KEYUP for virtual key 'A', with the repeat count and transition bits.
        ok := PostMessageW(window, ifx down then 0x100 else 0x101, 0x41, ifx down then 1 else 0xc000_0001);
        assert(ok != 0, "PostMessageW key");
    }

    post_click :: (window: HWND, down: bool) {
        // WM_LBUTTONDOWN / WM_LBUTTONUP at client (40, 40).
        ok := PostMessageW(window, ifx down then 0x201 else 0x202, ifx down then 1 else 0, (40 << 16) | 40);
        assert(ok != 0, "PostMessageW click");
    }

    close_like_the_user :: (window: HWND) {
        // The close button sends WM_SYSCOMMAND SC_CLOSE, which becomes a sent WM_CLOSE.
        SendMessageW(window, 0x112, 0xf060, 0);
    }

    GetCursor :: () -> HCURSOR #foreign user32;

    show_hand :: (window: HWND) {
        SetCursor(LoadCursorW(null, cast(*s16) IDC_HAND));
    }

    move_pointer :: (window: HWND) {
        // What Windows sends as the pointer moves over the client area: WM_SETCURSOR with
        // HTCLIENT and WM_MOUSEMOVE.
        SendMessageW(window, 0x20, cast(u64) window, 1 | (0x200 << 16));
    }

    hand_shown :: (window: HWND) -> bool {
        return GetCursor() == LoadCursorW(null, cast(*s16) IDC_HAND);
    }
} else #if OS == .LINUX {
    #import "X11";

    INVALID_WINDOW_HANDLE :: cast(Window) 0;

    post_key :: (window: Window, down: bool) {
        event: XEvent;
        event.xkey.type = ifx down then KeyPress else KeyRelease;
        event.xkey.display = x_global_display;
        event.xkey.window = window;
        event.xkey.root = XDefaultRootWindow(x_global_display);
        event.xkey.keycode = XKeysymToKeycode(x_global_display, 0x61); // XK_a
        event.xkey.same_screen = True;
        event.xkey.x = 10;
        event.xkey.y = 10;
        sent := XSendEvent(x_global_display, window, True, ifx down then KeyPressMask else KeyReleaseMask, *event);
        assert(sent != 0, "XSendEvent key");
        XFlush(x_global_display);
    }

    post_click :: (window: Window, down: bool) {
        event: XEvent;
        event.xbutton.type = ifx down then ButtonPress else ButtonRelease;
        event.xbutton.display = x_global_display;
        event.xbutton.window = window;
        event.xbutton.root = XDefaultRootWindow(x_global_display);
        event.xbutton.button = 1;
        event.xbutton.same_screen = True;
        event.xbutton.x = 40;
        event.xbutton.y = 40;
        sent := XSendEvent(x_global_display, window, True, ifx down then ButtonPressMask else ButtonReleaseMask, *event);
        assert(sent != 0, "XSendEvent click");
        XFlush(x_global_display);
    }

    close_like_the_user :: (window: Window) {
        // What a window manager sends when the user closes a window: WM_DELETE_WINDOW.
        event: XEvent;
        event.xclient.type = ClientMessage;
        event.xclient.window = window;
        event.xclient.message_type = x_global_wm_protocols;
        event.xclient.format = 32;
        event.xclient.data.l[0] = cast(s64) x_global_wm_delete_window;
        event.xclient.data.l[1] = CurrentTime;
        sent := XSendEvent(x_global_display, window, False, NoEventMask, *event);
        assert(sent != 0, "XSendEvent close");
        XFlush(x_global_display);
    }

    // The cursor the server shows, through XFixes: each cursor has its own serial number.
    xfixes :: #system_library "Xfixes";

    XFixesCursorImage :: struct {
        x: s16;
        y: s16;
        width: u16;
        height: u16;
        xhot: u16;
        yhot: u16;
        cursor_serial: u64;
        pixels: *u64;
        atom: u64;
        name: *u8;
    }

    XFixesGetCursorImage :: (display: *Display) -> *XFixesCursorImage #foreign xfixes;
    hand_cursor: Cursor;
    hand_serial: u64;

    shown_serial :: () -> u64 {
        XSync(x_global_display, False);
        image := XFixesGetCursorImage(x_global_display);
        assert(image != null, "XFixesGetCursorImage");
        serial := image.cursor_serial;
        XFree(image);
        return serial;
    }

    show_hand :: (window: Window) {
        before := shown_serial();
        if !hand_cursor hand_cursor = XCreateFontCursor(x_global_display, 60); // hand2
        XDefineCursor(x_global_display, window, hand_cursor);
        XWarpPointer(x_global_display, 0, window, 0, 0, 0, 0, 20, 20);
        // The server may show the new cursor a moment later; wait for it to change.
        hand_serial = shown_serial();
        for 1..200 {
            if hand_serial != before break;
            sleep_milliseconds(5);
            hand_serial = shown_serial();
        }
    }

    move_pointer :: (window: Window) {
        XWarpPointer(x_global_display, 0, window, 0, 0, 0, 0, 30, 30);
        XFlush(x_global_display);
    }

    hand_shown :: (window: Window) -> bool {
        return shown_serial() == hand_serial;
    }
}
