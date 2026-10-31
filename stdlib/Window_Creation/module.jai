// Opens native windows (Cocoa, Win32 or X11) with an OpenGL surface.
//
// `create_window` makes one; `get_dimensions`, `get_mouse_pointer_position` and `toggle_fullscreen`
// query and change it. Read input with the `Input` module. The module parameter `DEFAULT_MSAA`
// sets the default sample count.
//
// ```jai,no_run
// #import "Window_Creation";
// #import "Input";
//
// window := create_window(800, 600, "hello");
// assert(window != WT.INVALID_WINDOW);
// while true {
//     update_window_events();
//     for events_this_frame if it.type == .QUIT return;
// }
// ```

#module_parameters (DEFAULT_MSAA: s32 = 4);

WT :: #import "Window_Type";

// The native window handle. Its type depends on the target: an `X11.Window` (an integer) on Linux,
// an `HWND` on Windows, and a pointer on macOS (`*NSWindow`) and Android. So "no window" is `0`
// in one place and `null` in another, and `window == null` does not compile on Linux.
//
// To stay portable, never write the literal. A variable of the type starts as "no window" on every
// target, and `WT.INVALID_WINDOW` is the same value as a constant to compare against or assign.
// Handles compare with `==` on all targets.
//
// ```jai,skip_on(wasm)
// no_window: Window_Type;
// assert(no_window == WT.INVALID_WINDOW);
//
// window := no_window;
// if window == WT.INVALID_WINDOW window = no_window;
// ```
Window_Type :: WT.Window_Type;

#if OS == .LINUX {
    // The position and size to restore after leaving full screen; `toggle_fullscreen` fills it in.
    Saved_Window_Info :: struct {}
    window_minimized := false;

    #load "../internal/window/x11-queries.jai";
    #load "../internal/window/x11-create.jai";
} else #if OS == .WINDOWS {
    // The Win32 window style of a normal window.
    WINDOWED_STYLE: u32 : Windows.WS_OVERLAPPEDWINDOW;
    FULLSCREEN_STYLE: u32 : Windows.WS_VISIBLE | Windows.WS_POPUP;
    SECONDARY_WINDOW_STYLE: u32 : Windows.WS_OVERLAPPED | Windows.WS_CAPTION | Windows.WS_SYSMENU | Windows.WS_THICKFRAME;

    // The position and size to restore after leaving full screen; `toggle_fullscreen` fills it in.
    Saved_Window_Info :: struct {
        rect: Windows.RECT;
        style: s32;
        ex_style: s32;
    }

    #load "../internal/window/win32-queries.jai";
    #load "../internal/window/win32-create.jai";
} else #if OS == .MACOS {
    application_initialized := false;

    // The position and size to restore after leaving full screen; `toggle_fullscreen` fills it in.
    Saved_Window_Info :: struct {
        mask: NSWindowStyleMask;
        window_frame: Foundation.NSRect;
    }

    #load "../internal/window/cocoa-queries.jai";
} else #if OS == .ANDROID {
    // The position and size to restore after leaving full screen; `toggle_fullscreen` fills it in.
    Saved_Window_Info :: struct {}
} else {
    #assert false, "Window_Creation has no declarations for this target";
}

#scope_module
DEFAULT_WINDOW_CREATION_COLOR :: float.[0.15, 0.15, 0.2];

#if OS == .LINUX {
    #import "X11";
    Connection :: #import "internal/window/connection";
}

#if OS == .WINDOWS {
    Windows :: #import "Windows";
}

#if OS == .MACOS {
    ObjC :: #import "Objective_C/AppKit";
    Foundation :: #import "Objective_C";
    #import "Objective_C/AppKit";
    #import "macOS";
}
