// Reviewed public layout and canonical cross-module identities.
#import "Basic";
A :: #import "Objective_C/AppKit";
Same :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

probe :: #library "appkit_probe";
application_slot :: () -> *void #foreign probe "appkit_probe_application_slot";
runloop_value :: () -> *void #foreign probe "appkit_probe_runloop_value";
check_view :: (pointer: *void) -> s32 #foreign probe "appkit_probe_view";
no_application :: () -> s32 #foreign probe "appkit_probe_no_application";
key_event :: () -> *A.NSEvent #foreign probe "appkit_probe_key_event";
level :: (key: s32) -> s32 #foreign probe "appkit_probe_level";

#assert size_of(A.NSView) == 152;
#assert size_of(A.NSApplication) == size_of(O.NSObject);
#assert size_of(A.NSEvent) == size_of(O.NSObject);
#assert size_of(A.NSOpenGLContext) == size_of(O.NSObject);
#assert size_of(A.NSOpenGLPixelFormat) == size_of(O.NSObject);
#assert A.NSOpenGLContext == Same.NSOpenGLContext;
#assert type_info(A.NSOpenGLContext).name == "NSOpenGLContext";
#assert A.NSWindowStyleMaskTitled == 1;
#assert A.NSBackingStoreBuffered == 2;
#assert A.NSEventMaskKeyDown == 1 << 10;
#assert A.NSEventMaskAny == 0xffffffffffffffff;
#assert A.NSOpenGLContextParameterSurfaceSurfaceVolatile == 306;
#assert A.NSApplicationPresentationDisableCursorLocationAssistance == 1 << 12;
#assert A.NSOpenGLProfileVersion4_1Core == 0x4100;

main :: () {
    // Inspect compatibility storage only; never dispatch on this Jai record.
    compatibility_storage: A.NSView;

    assert(!compatibility_storage.super.isa);
    for byte: compatibility_storage.pad assert(byte == 0);

    found_super, found_pad: bool;

    for member: type_info(A.NSView).members {
        if member.name == "super" {
            found_super = true;
            assert(member.offset_in_bytes == 0);
        }

        if member.name == "pad" {
            found_pad = true;
            assert(member.offset_in_bytes == size_of(O.NSObject));
        }
    }

    assert(found_super && found_pad);
    // NSView allocation is independent of NSApplication/window/activation state.
    assert(!A.NSApp);
    A.init_app_kit();
    assert(!A.NSApp);
    levels := s32.[
        A.kCGBaseWindowLevel, A.kCGMinimumWindowLevel, A.kCGDesktopWindowLevel,
        A.kCGBackstopMenuLevel, A.kCGNormalWindowLevel, A.kCGFloatingWindowLevel,
        A.kCGTornOffMenuWindowLevel, A.kCGDockWindowLevel, A.kCGMainMenuWindowLevel,
        A.kCGStatusWindowLevel, A.kCGModalPanelWindowLevel, A.kCGPopUpMenuWindowLevel,
        A.kCGDraggingWindowLevel, A.kCGScreenSaverWindowLevel, A.kCGMaximumWindowLevel,
        A.kCGOverlayWindowLevel, A.kCGHelpWindowLevel, A.kCGUtilityWindowLevel,
        A.kCGDesktopIconWindowLevel, A.kCGCursorWindowLevel, A.kCGAssistiveTechHighWindowLevel,
    ];

    assert(levels.count == cast(s64) A.kCGNumberOfWindowLevelKeys);
    for value, index: levels assert(value == level(cast(s32) index));

    A.kCGBaseWindowLevel = 999;
    A.kCGNormalWindowLevel = 73;
    A.init_app_kit();
    assert(Same.kCGBaseWindowLevel == 999 && Same.kCGNormalWindowLevel == 73);
    A.kCGBaseWindowLevel = 0;
    A.init_app_kit();
    assert(A.kCGNormalWindowLevel == level(cast(s32) A.kCGNormalWindowLevelKey));
    assert(A.NSOpenGLContext.update(null) == null);
    assert(A.NSApplication.finishLaunching(null) == null);
    assert(A.NSApplication.setDelegate(null, null) == null);
    assert(A.NSApplication.setMainMenu(null, null) == null);
    view := O.objc_new(A.NSView);

    assert(view != null);
    defer O.release(view);
    rect := O.NSMakeRect(11, 23, 71, 89);

    A.NSView.setFrame(view, rect);
    result := A.NSView.frame(view);

    assert(result.origin.x == 11 && result.origin.y == 23);
    assert(result.size.width == 71 && result.size.height == 89);
    bounds := A.NSView.bounds(view);

    assert(bounds.size.width == 71 && bounds.size.height == 89);
    backing_rect := A.NSView.convertRectToBacking(view, rect);
    restored_rect := A.NSView.convertRectFromBacking(view, backing_rect);

    assert(restored_rect.origin.x == rect.origin.x && restored_rect.origin.y == rect.origin.y);
    assert(restored_rect.size.width == rect.size.width && restored_rect.size.height == rect.size.height);
    assert(A.NSView.mouse_inRect(view, O.NSMakePoint(2, 3), bounds) == O.YES);
    assert(A.NSView.mouse_inRect(view, O.NSMakePoint(-3, -3), bounds) == O.NO);
    assert(A.NSView.isInFullScreenMode(view) == O.NO);
    point := O.NSMakePoint(2, 3);
    backed := A.NSView.convertPointToBacking(view, point);
    restored := A.NSView.convertPointFromBacking(view, backed);

    assert(restored.x == point.x && restored.y == point.y);
    child := O.objc_new(A.NSView);

    assert(child != null);
    defer O.release(child);
    A.NSView.addSubView(view, child);
    assert(check_view(view) == 1);
    assert(!A.NSApp);
    assert(no_application() == 1);
    assert(cast(*void) *A.NSApp == application_slot());
    assert(A.NSDefaultRunLoopMode != null);
    assert(cast(*void) A.NSDefaultRunLoopMode == runloop_value());
    event := key_event();

    assert(event != null);
    defer O.release(event);
    assert(A.NSEvent.type(event) == A.NSEventTypeKeyDown);
    assert(A.NSEvent.keyCode(event) == 42);
    assert(A.NSEvent.modifierFlags(event) == A.NSEventModifierFlagShift);
    assert(A.NSEvent.isARepeat(event) == O.NO);
    assert(O.to_string(A.NSEvent.characters(event)) == "x");
    assert(O.to_string(A.NSEvent.charactersIgnoringModifiers(event)) == "x");
    assert(A.NSOpenGLContext.currentContext() == null);
    assert(no_application() == 1);
}
