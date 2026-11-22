// Generated from docs/specs/stdlib/modules/nvtt.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#if OS == .WINDOWS {
    #import "Windows";
}

#if OS == .MACOS {
    #library,system,link_always "c++"; @native(external)
} else #if OS == .LINUX {
    #library,system,link_always "stdc++"; @native(external)
}

libnvtt :: #library,system "nvtt"; @native(external)
#scope_export

// Types and layouts

// Callback and opaque types

// Aliases and constants

// Native entry points

// Convenience operations

// Each platform's own declarations, parsed only when compiling for it.
#if OS == .LINUX {
    #load "declarations/linux.jai";
}

#if OS == .MACOS {
    #load "declarations/macos.jai";
}

#if OS == .WINDOWS {
    #load "declarations/windows.jai";
}
