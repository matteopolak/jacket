#module_parameters (DEFAULT_MSAA: s32 = 4);

WT :: #import "Window_Type";

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
    #assert false, "Window_Creation has no reviewed declarations for this target";
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
