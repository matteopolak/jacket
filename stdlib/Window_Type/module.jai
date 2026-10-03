#if OS == .LINUX {
    Window_Type :: X11.Window;
    INVALID_WINDOW: Window_Type : 0;
} else #if OS == .WINDOWS {
    Window_Type :: Windows.HWND;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .MACOS {
    Window_Type :: *ObjC.NSWindow;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .ANDROID {
    Window_Type :: *Android.ANativeWindow;
    INVALID_WINDOW: Window_Type : null;
} else #if OS == .PS5 || OS == .NN_SWITCH2 {
    Window_Type :: *void;
    INVALID_WINDOW: Window_Type : null;
} else {
    #assert false, "Window_Type has no reviewed native handle for this target";
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
