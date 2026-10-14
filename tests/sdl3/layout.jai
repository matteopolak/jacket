#import "Extensions/SDL3";
#if OS == .MACOS {
    #if CPU == .X64 { #load "layout-macos-x64.jai"; }
    else { #load "layout-macos-arm64.jai"; }
} else #if OS == .LINUX {
    #if CPU == .X64 { #load "layout-linux-x64.jai"; }
    else { #load "layout-linux-arm64.jai"; }
} else #if OS == .WINDOWS {
    #if CPU == .X64 { #load "layout-windows-x64.jai"; }
    else { #load "layout-windows-arm64.jai"; }
}
main :: () {}
