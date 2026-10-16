// Reviewed Forbear regression: canonical using/#as path, no alternate overload.
A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";

view_control :: (value: *A.NSView) -> O.id {
    return value;
}

window_conversion :: (value: *A.NSWindow) -> O.id {
    return value;
}

screen_initializer :: (value: *A.NSWindow, screen: *A.NSScreen) -> *A.NSWindow {
    return A.NSWindow.initWithContentRect(value, .{}, .NSWindowStyleMaskTitled,
                                        .NSBackingStoreBuffered, 0, screen);
}

main :: () {}
