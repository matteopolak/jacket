// Execute only under the coordinator's current serialized GUI lease.
#import "Basic";
A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

P :: #library "appkit_ui_probe";
begin :: () -> *void #foreign P "appkit_ui_begin";
finish :: (state: *void) -> s32 #foreign P "appkit_ui_finish";
close :: (window: *A.NSWindow) #foreign P "appkit_ui_close";
clear :: () #foreign P "appkit_ui_clear";
menu_matches :: (menu: *A.NSMenu) -> s32 #foreign P "appkit_ui_menu";
context_matches :: (gl_context: *A.NSOpenGLContext, view: *A.NSView) -> s32 #foreign P "appkit_ui_context";

smoke :: () -> bool {
    A.init_app_kit();
    app := A.NSApplication.sharedApplication();
    if !app || app != A.NSApp return false;
    if !A.NSApplication.setActivationPolicy(app, .NSApplicationActivationPolicyAccessory) return false;

    A.NSApplication.finishLaunching(app); // Reviewed id slot is ignored.
    menu := O.objc_new(A.NSMenu);
    if !menu return false;

    defer O.release(menu);
    A.NSApplication.setMainMenu(app, menu); // Check real effect, not the id slot.
    if menu_matches(menu) != 1 return false;

    A.NSApplication.setMainMenu(app, null);
    window := A.NSWindow.initWithContentRect(O.objc_alloc(A.NSWindow),
        O.NSMakeRect(20, 20, 64, 64), .NSWindowStyleMaskBorderless, .NSBackingStoreBuffered, O.NO);
    if !window return false;

    A.NSWindow.setReleasedWhenClosed(window, O.NO);

    defer {
        close(window);
        O.release(window);
    }

    view := A.NSWindow.contentView(window);
    if !view return false;

    attributes := A.NSOpenGLPixelFormatAttribute.[A.NSOpenGLPFAOpenGLProfile,
        A.NSOpenGLProfileVersion4_1Core, A.NSOpenGLPFADoubleBuffer, 0];
    format := A.NSOpenGLPixelFormat.initWithAttributes(O.objc_alloc(A.NSOpenGLPixelFormat), attributes.data);

    if !format return false;

    defer O.release(format);
    gl_context := A.NSOpenGLContext.initWithFormat(O.objc_alloc(A.NSOpenGLContext), format, null);
    if !gl_context return false;

    defer {
        clear();
        A.NSOpenGLContext.setView(gl_context, null);
        O.release(gl_context);
    }

    A.NSOpenGLContext.setView(gl_context, view);
    A.NSOpenGLContext.update(gl_context); // Reviewed id slot is ignored.
    A.NSOpenGLContext.makeCurrentContext(gl_context);
    if context_matches(gl_context, view) != 1 return false;

    A.NSOpenGLContext.flushBuffer(gl_context);

    return true;
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    assert(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    state := begin();

    assert(state != null);
    okay := smoke();
    restored := finish(state);

    assert(restored == 1);
    assert(okay);
}
