// Independent SDK injection support, not an Input adapter or compatibility API.
// Callers must obtain the serialized native-runtime lease before execution.
#assert OS == .MACOS, "this fixture requires the reviewed macOS Input hook and AppKit";

O :: #import "Objective_C";
W :: #import "Window_Creation";

Owned_Window :: struct {
    window: W.Window_Type;
    released: bool;
}

owned_windows: [..] Owned_Window;

owned_native_window :: (width: int, height: int, title: string) -> W.Window_Type {
    window := W.create_window(width, height, title);
    if window != null array_add(*owned_windows, .{ window, false });
    return window;
}

dispose_native_windows :: () {
    // Main defers this before acquiring windows. On fatal process termination
    // the supervisor reaps the child; WindowServer then retires its resources.
    for item: owned_windows if !item.released native_message(item.window, "close");

    Input.update_window_events();

    for *item: owned_windows {
        if !item.released {
            native_message(item.window, "release");
            item.released = true;
        }
    }

    array_free(owned_windows);
    owned_windows = .{};
}

native_message :: (receiver: *void, name: string) {
    Call :: #type (receiver: *void, selector: O.Selector) #c_call;
    call := cast(Call) O.objc_msgSend;
    call(receiver, O.selector(name));
}

native_integer :: (receiver: *void, name: string) -> s64 {
    Call :: #type (receiver: *void, selector: O.Selector) -> s64 #c_call;
    call := cast(Call) O.objc_msgSend;
    return call(receiver, O.selector(name));
}

native_key :: (window: W.Window_Type, down: bool) {
    Pool :: #type (receiver: *void, selector: O.Selector) -> *void #c_call;
    object := cast(Pool) O.objc_msgSend;
    pool := object(O.objc_getClass("NSAutoreleasePool"), O.selector("new"));

    assert(pool != null);
    defer native_message(pool, "release");
    // NSEventTypeKeyDown/Up, physical A, NSString character payload, BOOL repeat.
    Make :: #type (receiver: *void, selector: O.Selector, type: u64, point: O.NSPoint,
                   flags: u64, time: float64, number: s64, context: *void,
                   characters: *void, unmodified: *void, repeat: O.BOOL, code: u16) -> *void #c_call;
    make := cast(Make) O.objc_msgSend;
    text := O.NSString.getTempString("a");
    event := make(O.objc_getClass("NSEvent"), O.selector("keyEventWithType:location:modifierFlags:timestamp:windowNumber:context:characters:charactersIgnoringModifiers:isARepeat:keyCode:"),
                  ifx down then 10 else 11, .{ 10, 10 }, 0, 0,
                  native_integer(window, "windowNumber"), null, text, text, O.NO, 0);

    assert(event != null, "native key factory failed");
    Post :: #type (receiver: *void, selector: O.Selector, event: *void, front: O.BOOL) #c_call;
    post := cast(Post) O.objc_msgSend;
    application := object(O.objc_getClass("NSApplication"), O.selector("sharedApplication"));

    post(application, O.selector("postEvent:atStart:"), event, O.NO);
}

has_key :: (window: W.Window_Type, key: Input.Key_Code, down: bool) -> bool {
    for event: Input.events_this_frame {
        if event.window == window && event.type == .KEYBOARD && event.key_code == key && (event.key_pressed != 0) == down return true;
    }

    return false;
}

has_window_event :: (window: W.Window_Type, kind: Input.Window_Event_Type) -> bool {
    for event: Input.events_this_frame {
        if event.window == window && event.type == .WINDOW && event.window_event_type == kind return true;
    }

    return false;
}

close_native_window :: (window: W.Window_Type) {
    native_message(window, "close");
    Input.update_window_events();
    found := false;
    for event: Input.events_this_frame if event.type == .QUIT && event.window == window found = true;

    assert(found, "native close lost its public Event.window identity");
    native_message(window, "release");
    for *item: owned_windows if item.window == window item.released = true;
}
