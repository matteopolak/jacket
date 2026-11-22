// Generated from docs/specs/stdlib/modules/linux.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "POSIX";

libc :: #library,system "c"; @native(external)
#scope_export

// Types and layouts

// Aliases and constants

// Native storage

// Native entry points

// Convenience operations

// Each platform's own declarations, parsed only when compiling for it.
#if OS == .LINUX {
    #load "declarations/linux.jai";
}
