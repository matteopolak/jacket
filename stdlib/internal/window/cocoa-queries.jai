// Opens a window and returns it, or a null window when it could not be made.
//
// - width: in window units.
// - height: in window units.
// - window_name: the title.
// - window_x: where the top left corner goes, in screen units (or inside `parent`). Leave it at -1 on
//   macOS to centre the window and on Windows for the top left of the usable screen; Linux has no such
//   default and starts at 0.
// - window_y: the same for the vertical position.
// - parent: a window to put this one in. Null makes a top-level window.
// - background_color_rgb: red, green and blue from 0 to 1, shown before the first frame is drawn.
// - wanted_msaa: samples per pixel for smoothing edges. The default is the module parameter
//   `DEFAULT_MSAA`.
create_window :: (width: int, height: int, window_name: string, window_x := -1, window_y := -1, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> *ObjC.NSWindow {
    init_mac_app();
    window := cast(*ObjC.NSWindow) Cocoa.create_content_window(cast(s32) width, cast(s32) height, window_name, cast(s32) window_x, cast(s32) window_y);
    if window && window_x == -1 && window_y == -1 ObjC.NSWindow.center(window);
    return window;
}

init_mac_app :: () {
    application_initialized = Cocoa.initialize_application();
}

swap_view_buffers :: (win: *ObjC.NSWindow) {
    Cocoa.swap_window_buffers(win);
}

// Where the window's drawing area is on the screen and how big it is: `x, y, width, height, success`.
//
// - right_handed: counts `y` up from the bottom of the screen instead of down from the top.
get_dimensions :: (window: *ObjC.NSWindow, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true) {
    if !window return 0, 0, 0, 0, false;

    rect := Cocoa.content_screen_rect(window);
    y := rect.origin.y;

    if !right_handed y = Cocoa.screen_top() - y - rect.size.height;

    return cast(s32) rect.origin.x, cast(s32) y, cast(s32) rect.size.width, cast(s32) rect.size.height, true;
}

/// Device pixels per window coordinate unit: 2 on a Retina display, 1 elsewhere.
get_pixel_scale :: (window: *ObjC.NSWindow) -> float {
    if !window return 1;
    return Cocoa.backing_scale(window);
}

// Where the mouse is, in window coordinates: `x, y, success`.
//
// - right_handed: counts `y` up from the bottom of the window instead of down from the top.
get_mouse_pointer_position :: (window: *ObjC.NSWindow, right_handed: bool) -> x: int, y: int, success: bool {
    if !window return 0, 0, false;
    point := Cocoa.local_mouse_location(window);
    if !right_handed point.y = Cocoa.content_size(window).height - point.y;
    return cast(int) point.x, cast(int) point.y, true;
}

// Where the mouse is, in window coordinates: `x, y, success`.
//
// - right_handed: counts `y` up from the bottom of the window instead of down from the top.
get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool {
    point := Cocoa.mouse_location();
    if !right_handed point.y = Cocoa.screen_top() - point.y;
    return cast(int) point.x, cast(int) point.y, true;
}

// Switches the window between full screen and its normal size. Returns whether it worked and the new size.
//
// - desire_fullscreen: true to go full screen, false to go back.
// - saved: where the old position and size are kept while full screen, to restore them on the way
//   back.
toggle_fullscreen :: (win: *ObjC.NSWindow, desire_fullscreen: bool, saved: *Saved_Window_Info) -> success: bool, width: s32, height: s32 {
    if !win || !saved return false, 0, 0;

    current := ObjC.NSWindow.styleMask(win);
    fullscreen := (current & .NSWindowStyleMaskFullScreen) != .NSWindowStyleMaskBorderless;

    if desire_fullscreen != fullscreen {
        if desire_fullscreen {
            saved.mask = current;
            saved.window_frame = Cocoa.result_message(win, "frame", Foundation.NSRect);
        }

        ObjC.NSWindow.toggleFullScreen(win, null);
    }

    size := Cocoa.content_size(win);

    // AppKit transitions asynchronously; these are actual current dimensions.
    return true, cast(s32) size.width, cast(s32) size.height;
}

#scope_module
Cocoa :: #import "internal/window/cocoa";
