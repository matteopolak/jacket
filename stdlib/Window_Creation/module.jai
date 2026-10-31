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
    Saved_Window_Info :: struct {}
    window_minimized := false;

    #load "../internal/window/x11-queries.jai";
    #load "../internal/window/x11-create.jai";
} else #if OS == .WINDOWS {
    WINDOWED_STYLE: u32 : Windows.WS_OVERLAPPEDWINDOW;
    FULLSCREEN_STYLE: u32 : Windows.WS_VISIBLE | Windows.WS_POPUP;
    SECONDARY_WINDOW_STYLE: u32 : Windows.WS_OVERLAPPED | Windows.WS_CAPTION | Windows.WS_SYSMENU | Windows.WS_THICKFRAME;

    Saved_Window_Info :: struct {
        rect: Windows.RECT;
        style: s32;
        ex_style: s32;
    }

    #load "../internal/window/win32-queries.jai";
    #load "../internal/window/win32-create.jai";
} else #if OS == .MACOS {
    application_initialized := false;

    Saved_Window_Info :: struct {
        mask: NSWindowStyleMask;
        window_frame: Foundation.NSRect;
    }

    #load "../internal/window/cocoa-queries.jai";
} else #if OS == .ANDROID {
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
