A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

#assert type_info(A.NSWindow).name == "NSWindow";
#assert type_info(A.NSResponder).name == "NSResponder";
#assert size_of(A.NSWindow) == size_of(O.NSObject);
#assert size_of(A.NSResponder) == size_of(O.NSObject);
#assert cast(u64) A.NSWindowStyleMaskFullSizeContentView == 32768;
#assert cast(u64) A.NSWindowCollectionBehavior.FullScreenAllowsTiling == 2048;
#assert cast(u64) A.NSBackingStoreBuffered == 2;

libc :: #library,system "c";
abort :: () #foreign libc;
libobjc :: #library,system "objc";
native_alloc :: (class: O.Class, selector: O.Selector) -> *void #foreign libobjc "objc_msgSend";
native_init_frame :: (self: *void, selector: O.Selector, frame: O.NSRect) -> *void #foreign libobjc "objc_msgSend";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    O.init_objective_c();
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    responder := O.objc_new(A.NSResponder);

    check(responder != null);
    O.release(responder);

    // NSView does not require creating or displaying an NSWindow. This checks
    // the reviewed frame alias and the actual aggregate message return ABI.
    view_class := O.objc_getClass("NSView");

    check(view_class != null);
    storage := native_alloc(view_class, O.selector("alloc"));

    check(storage != null);
    expected: O.NSRect;
    expected.origin.x = 13;
    expected.origin.y = 29;
    expected.size.width = 127;
    expected.size.height = 61;
    view := cast(type_of(A.NSWindow.contentView(null))) native_init_frame(storage,
        O.selector("initWithFrame:"), expected);

    check(view != null);
    actual := A.NSWindow.frame(view);

    check(cast(*void) A.NSWindow.frame == cast(*void) view.frame);

    check(actual.origin.x == expected.origin.x && actual.origin.y == expected.origin.y);
    check(actual.size.width == expected.size.width && actual.size.height == expected.size.height);
    O.release(view);

    window_class := O.objc_getClass("NSWindow");

    check(window_class != null);

    for selector: string.["initWithContentRect:styleMask:backing:defer:",
                         "initWithContentRect:styleMask:backing:defer:screen:",
                         "setTitle:", "setCollectionBehavior:", "contentView",
                         "makeFirstResponder:", "toggleFullScreen:"] {
        check(O.class_getInstanceMethod(window_class, O.selector(selector)) != null);
    }
}
