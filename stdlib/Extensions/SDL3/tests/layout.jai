#import "Extensions/SDL3";

#if OS == .MACOS {
    #if CPU == .X64 {
        #load "driven/layout-macos-x64.jai";
    } else {
        #load "driven/layout-macos-arm64.jai";
    }
} else #if OS == .LINUX {
    #if CPU == .X64 {
        #load "driven/layout-linux-x64.jai";
    } else {
        #load "driven/layout-linux-arm64.jai";
    }
} else #if OS == .WINDOWS {
    #if CPU == .X64 {
        #load "driven/layout-windows-x64.jai";
    } else {
        #load "driven/layout-windows-arm64.jai";
    }
}

main :: () {}
