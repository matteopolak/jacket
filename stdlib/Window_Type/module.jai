// The type of a native window handle, so code can pass windows around without importing the
// system's window module: an X11 `Window` on Linux, an `HWND` on Windows, an `NSWindow` pointer
// on macOS and an `ANativeWindow` pointer on Android.

#if OS == .LINUX {
    // The native window handle on this system; `INVALID_WINDOW` is the one that names no window.
    Window_Type :: X11.Window;
    INVALID_WINDOW: Window_Type : 0;
} else #if OS == .WINDOWS {
    // The native window handle on this system; `INVALID_WINDOW` is the one that names no window.
    Window_Type :: Windows.HWND;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .MACOS {
    // The native window handle on this system; `INVALID_WINDOW` is the one that names no window.
    Window_Type :: *ObjC.NSWindow;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .ANDROID {
    // The native window handle on this system; `INVALID_WINDOW` is the one that names no window.
    Window_Type :: *Android.ANativeWindow;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .PS5 || OS == .NN_SWITCH2 {
    // The native window handle on this system; `INVALID_WINDOW` is the one that names no window.
    Window_Type :: *void;
    INVALID_WINDOW: Window_Type : null;
} else {
    #assert false, "Window_Type has no native handle for this target";
}

#scope_module
#if OS == .LINUX {
    X11 :: #import "X11";
}

#if OS == .WINDOWS {
    Windows :: #import "Windows";
}

#if OS == .MACOS {
    ObjC :: #import "Objective_C/AppKit";
}

#if OS == .ANDROID {
    Android :: #import "Android";
}
