// AppKit objects remain native objects of the canonical binding types. This
// module owns native lifecycle glue; it never allocates class instance layouts.
initialize_application :: () -> bool {
    if application return true;

    loaded := application_load();
    if loaded == 0 {
        application_error("AppKit initialization failed");
        return false;
    }

    if result_message(native_class("NSThread"), "isMainThread", s8) == 0 {
        application_error("AppKit window operations require the main thread");
        return false;
    }

    candidate := result_message(native_class("NSApplication"), "sharedApplication", *void);
    if !candidate {
        application_error("NSApplication is unavailable");
        return false;
    }

    Policy :: #type (receiver: *void, selector: O.Selector, policy: s64) -> s8 #c_call;
    policy := cast(Policy) native_send;

    if result_message(candidate, "activationPolicy", s64) != 0 {
        activated := policy(candidate, O.selector("setActivationPolicy:"), 0);
        if activated == 0 {
            application_error("AppKit activation policy failed");
            return false;
        }
    }

    void_message(candidate, "finishLaunching");
    application = candidate;

    return true;
}

create_content_window :: (width: s32, height: s32, title: string, x: s32, y: s32) -> *A.NSWindow {
    if !initialize_application() return null;

    Basic.assert(width > 0 && height > 0, "native window dimensions must be positive");
    pool := autorelease_pool();

    defer release(pool);
    allocated := result_message(native_class("NSWindow"), "alloc", *A.NSWindow);

    Basic.assert(allocated != null, "NSWindow allocation failed");
    rect: O.NSRect;
    rect.origin = .{ cast(O.CGFloat) x, cast(O.CGFloat) y };
    rect.size = .{ cast(O.CGFloat) width, cast(O.CGFloat) height };
    window := A.NSWindow.initWithContentRect(allocated, rect,
        .NSWindowStyleMaskTitled | .NSWindowStyleMaskClosable | .NSWindowStyleMaskMiniaturizable | .NSWindowStyleMaskResizable,
        .NSBackingStoreBuffered, 0);
    if !window return null;

    A.NSWindow.setReleasedWhenClosed(window, 0);
    // The contextual binding uses the native void ABI; its id result is unused.
    A.NSWindow.setTitle(window, title);
    A.NSWindow.makeKeyAndOrderFront(window, null);

    return window;
}

close_window :: (window: *A.NSWindow) {
    if !window return;
    void_message(window, "close");
    release(window);
}

attach_render_context :: (window: *A.NSWindow, native: *void) {
    Basic.assert(window != null && native != null);
    argument_message(native, "setView:", A.NSWindow.contentView(window));
    set_association(window, *render_context_key, native, 1);
}

detach_render_context :: (window: *A.NSWindow, native: *void) {
    void_message(native, "clearDrawable");
    set_association(window, *render_context_key, null, 1);
}

swap_window_buffers :: (window: *A.NSWindow) {
    Basic.assert(window != null);
    native := get_association(window, *render_context_key);
    if !native {
        view := A.NSWindow.contentView(window);
        if argument_result_message(view, "respondsToSelector:", O.selector("openGLContext"), s8) != 0 {
            native = result_message(view, "openGLContext", *void);
        } else {
            native = result_message(O.objc_getClass("NSOpenGLContext"), "currentContext", *void);
            Basic.assert(native != null && result_message(native, "view", *void) == view, "window has no associated native OpenGL context");
        }
    }

    Basic.assert(native != null, "window has no native OpenGL context");
    void_message(native, "makeCurrentContext");
    void_message(native, "flushBuffer");
}

next_event :: () -> *void {
    if !initialize_application() return null;

    Next :: #type (receiver: *void, selector: O.Selector, mask: u64, until: *void, mode: *void, dequeue: s8) -> *void #c_call;
    next := cast(Next) native_send;
    date := result_message(O.objc_getClass("NSDate"), "distantPast", *void);

    return next(application, O.selector("nextEventMatchingMask:untilDate:inMode:dequeue:"), 0xffffffffffffffff, date, default_runloop_mode, 1);
}

dispatch_event :: (event: *void) {
    argument_message(application, "sendEvent:", event);
}

update_windows :: () {
    void_message(application, "updateWindows");
}

application_windows :: () -> *void {
    if !initialize_application() return null;
    return result_message(application, "windows", *void);
}

argument_result_message :: (receiver: *void, selector: string, value: $T, $Result: Type) -> Result {
    Send :: #type (receiver: *void, selector: O.Selector, value: T) -> Result #c_call;
    send := cast(Send) native_send;
    return send(receiver, O.selector(selector), value);
}

content_screen_rect :: (window: *A.NSWindow) -> O.NSRect {
    frame := result_message(window, "frame", O.NSRect);
    Convert :: #type (receiver: *A.NSWindow, selector: O.Selector, frame: O.NSRect) -> O.NSRect #c_call;
    convert: Convert;

    #if CPU == .X64 convert = cast(Convert) native_send_stret;
    else convert = cast(Convert) native_send;

    return convert(window, O.selector("contentRectForFrameRect:"), frame);
}

screen_top :: () -> O.CGFloat {
    screen := result_message(native_class("NSScreen"), "mainScreen", *void);
    Basic.assert(screen != null, "AppKit has no screen");
    rect := result_message(screen, "frame", O.NSRect);
    return rect.origin.y + rect.size.height;
}

mouse_location :: () -> O.NSPoint {
    return result_message(native_class("NSEvent"), "mouseLocation", O.NSPoint);
}

local_mouse_location :: (window: *A.NSWindow) -> O.NSPoint {
    return result_message(window, "mouseLocationOutsideOfEventStream", O.NSPoint);
}

content_size :: (window: *A.NSWindow) -> O.NSSize {
    view := A.NSWindow.contentView(window);
    rect := result_message(view, "bounds", O.NSRect);
    return rect.size;
}

result_message :: (receiver: *void, selector: string, $Result: Type) -> Result {
    Send :: #type (receiver: *void, selector: O.Selector) -> Result #c_call;
    send: Send;

    #if CPU == .X64 && size_of(Result) > 16 send = cast(Send) native_send_stret;
    else send = cast(Send) native_send;

    return send(receiver, O.selector(selector));
}

void_message :: (receiver: *void, selector: string) {
    Send :: #type (receiver: *void, selector: O.Selector) #c_call;
    send := cast(Send) native_send;
    send(receiver, O.selector(selector));
}

argument_message :: (receiver: *void, selector: string, value: $T) {
    Send :: #type (receiver: *void, selector: O.Selector, value: T) #c_call;
    send := cast(Send) native_send;
    send(receiver, O.selector(selector), value);
}

autorelease_pool :: () -> *void {
    result := result_message(native_class("NSAutoreleasePool"), "new", *void);
    Basic.assert(result != null, "autorelease pool allocation failed");
    return result;
}

release :: (object: *void) {
    void_message(object, "release");
}

native_string :: (text: string) -> *void {
    result := O.NSString.getTempString(text);

    Basic.assert(result != null, "native string is not valid UTF-8");

    return result;
}

#scope_module
application_error :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    Basic.assert(false, message);
}

A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";
Basic :: #import "Basic";

application: *void;
render_context_key: u64;

objc :: #library,system "objc";
appkit :: #library,system "AppKit";
foundation :: #library,system "Foundation";
default_runloop_mode: *void #elsewhere foundation "NSDefaultRunLoopMode";
application_load :: () -> s8 #foreign appkit "NSApplicationLoad";
native_class :: (name: *u8) -> *void #foreign objc "objc_getClass";
native_send :: () #foreign objc "objc_msgSend";
set_association :: (object: *void, key: *void, value: *void, policy: u64) #foreign objc "objc_setAssociatedObject";
get_association :: (object: *void, key: *void) -> *void #foreign objc "objc_getAssociatedObject";
#if CPU == .X64 native_send_stret :: () #foreign objc "objc_msgSend_stret";
