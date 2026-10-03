#if OS == .MACOS {
    #load "../internal/clipboard/cocoa.jai";
} else #if OS == .WINDOWS {
    #load "../internal/clipboard/win32.jai";
} else #if OS == .LINUX {
    #load "../internal/clipboard/linux.jai";
} else {
    #assert false, "Clipboard requires macOS, Windows or Linux/X11";
}

#scope_module
#import "Basic";
