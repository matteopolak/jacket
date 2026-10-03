create_window :: (width: int, height: int, window_name: string, window_x := -1, window_y := -1, parent: Window_Type = null, background_color_rgb := DEFAULT_WINDOW_CREATION_COLOR, wanted_msaa := DEFAULT_MSAA) -> *ObjC.NSWindow {
    init_mac_app();
    window := Cocoa.create_content_window(cast(s32) width, cast(s32) height, window_name, cast(s32) window_x, cast(s32) window_y);
    if window && window_x == -1 && window_y == -1 ObjC.NSWindow.center(window);
    return window;
}

init_mac_app :: () {
    application_initialized = Cocoa.initialize_application();
}

swap_view_buffers :: (win: *ObjC.NSWindow) {
    Cocoa.swap_window_buffers(win);
}

get_dimensions :: (window: *ObjC.NSWindow, right_handed: bool) -> (x: s32, y: s32, width: s32, height: s32, success := true) {
    if !window return 0, 0, 0, 0, false;

    rect := Cocoa.content_screen_rect(window);
    y := rect.origin.y;

    if !right_handed y = Cocoa.screen_top() - y - rect.size.height;

    return cast(s32) rect.origin.x, cast(s32) y, cast(s32) rect.size.width, cast(s32) rect.size.height, true;
}

get_mouse_pointer_position :: (window: *ObjC.NSWindow, right_handed: bool) -> x: int, y: int, success: bool {
    if !window return 0, 0, false;
    point := Cocoa.local_mouse_location(window);
    if !right_handed point.y = Cocoa.content_size(window).height - point.y;
    return cast(int) point.x, cast(int) point.y, true;
}

get_mouse_pointer_position :: inline (right_handed := false) -> x: int, y: int, success: bool {
    point := Cocoa.mouse_location();
    if !right_handed point.y = Cocoa.screen_top() - point.y;
    return cast(int) point.x, cast(int) point.y, true;
}

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
