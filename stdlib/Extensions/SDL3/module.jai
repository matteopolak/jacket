// SDL3 3.4.18. A new C binding, independent of the SDL2 module.
// Official SDL copyright and zlib license are retained in LICENSE.txt.
// Regeneration and supported surfaces: docs/extensions/sdl3.md.
#assert(CPU == .X64 || CPU == .ARM64, "SDL3 supports x64 and arm64 release targets");
#assert(size_of(*void) == 8, "SDL3 release bindings require 64-bit pointers");
SDL3 :: #system_library "SDL3";

#if OS == .MACOS {
    #if CPU == .X64 {
        #load "macos-x64.jai";
    } else {
        #load "macos-arm64.jai";
    }

    #load "link-macos.jai";
} else #if OS == .LINUX {
    #if CPU == .X64 {
        #load "linux-x64.jai";
    } else {
        #load "linux-arm64.jai";
    }

    #load "link-linux.jai";
} else #if OS == .WINDOWS {
    #if CPU == .X64 {
        #load "windows-x64.jai";
    } else {
        #load "windows-arm64.jai";
    }

    #load "link-windows.jai";
} else {
    #assert(false, "Extensions/SDL3 supports macOS, Linux and Windows x64/arm64");
}

#load "macros.jai";
