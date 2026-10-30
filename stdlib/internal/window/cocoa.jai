// Windows cross this module as *void: Window_Creation gives them its own AppKit class type, which
// is the project's when a project ships its own Objective_C. This module owns native lifecycle
// glue; it never allocates class instance layouts.
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
    // Hardening, not a confirmed fix: AppKit may ignore requests such as toggleFullScreen: for an
    // app that is not active, and a program started from a shell is not active by itself. This
    // is not confirmed to fix the intermittent fullscreen-toggle timeout seen on CI under interp.
    argument_result_message(candidate, "activateIgnoringOtherApps:", cast(s8) 1, s8);
    application = candidate;

    return true;
}

create_content_window :: (width: s32, height: s32, title: string, x: s32, y: s32) -> *void {
    if !initialize_application() return null;

    Basic.assert(width > 0 && height > 0, "native window dimensions must be positive");
    pool := autorelease_pool();

    defer release(pool);
    allocated := result_message(native_class("NSWindow"), "alloc", *void);

    Basic.assert(allocated != null, "NSWindow allocation failed");
    rect: O.NSRect;
    rect.origin = .{ cast(O.CGFloat) x, cast(O.CGFloat) y };
    rect.size = .{ cast(O.CGFloat) width, cast(O.CGFloat) height };

    Init :: #type (receiver: *void, selector: O.Selector, rect: O.NSRect, style: u64, backing: u64, deferred: s8) -> *void #c_call;
    init := cast(Init) native_send;
    window := init(allocated, O.selector("initWithContentRect:styleMask:backing:defer:"), rect,
                   STYLE_TITLED | STYLE_CLOSABLE | STYLE_MINIATURIZABLE | STYLE_RESIZABLE, BACKING_BUFFERED, 0);

    if !window return null;

    argument_message(window, "setReleasedWhenClosed:", cast(s8) 0);
    argument_message(window, "setTitle:", native_string(title));
    argument_message(window, "makeKeyAndOrderFront:", cast(*void) null);

    return window;
}

close_window :: (window: *void) {
    if !window return;
    void_message(window, "close");
    release(window);
} @FreesArgument

attach_render_context :: (window: *void, native: *void) {
    Basic.assert(window != null && native != null);
    argument_message(native, "setView:", result_message(window, "contentView", *void));
    set_association(window, *render_context_key, native, 1);
}

detach_render_context :: (window: *void, native: *void) {
    void_message(native, "clearDrawable");
    set_association(window, *render_context_key, null, 1);
}

swap_window_buffers :: (window: *void) {
    Basic.assert(window != null);
    native := get_association(window, *render_context_key);
    if !native {
        view := result_message(window, "contentView", *void);
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

content_screen_rect :: (window: *void) -> O.NSRect {
    frame := result_message(window, "frame", O.NSRect);
    Convert :: #type (receiver: *void, selector: O.Selector, frame: O.NSRect) -> O.NSRect #c_call;
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

local_mouse_location :: (window: *void) -> O.NSPoint {
    return result_message(window, "mouseLocationOutsideOfEventStream", O.NSPoint);
}

content_size :: (window: *void) -> O.NSSize {
    view := result_message(window, "contentView", *void);
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
} @FreesArgument

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

O :: #import "internal/bindings/objc-api";
Basic :: #import "Basic";

application: *void;

STYLE_TITLED :: 1;
STYLE_CLOSABLE :: 2;
STYLE_MINIATURIZABLE :: 4;
STYLE_RESIZABLE :: 8;
BACKING_BUFFERED :: 2;
render_context_key: u64;

objc :: #library,system "objc"; @native(external)
appkit :: #library,system "AppKit"; @native(external)
foundation :: #library,system "Foundation"; @native(external)
default_runloop_mode: *void #elsewhere foundation "NSDefaultRunLoopMode";
application_load :: () -> s8 #foreign appkit "NSApplicationLoad";
native_class :: (name: *u8) -> *void #foreign objc "objc_getClass";
native_send :: () #foreign objc "objc_msgSend";
set_association :: (object: *void, key: *void, value: *void, policy: u64) #foreign objc "objc_setAssociatedObject";
get_association :: (object: *void, key: *void) -> *void #foreign objc "objc_getAssociatedObject";
#if CPU == .X64 native_send_stret :: () #foreign objc "objc_msgSend_stret";
