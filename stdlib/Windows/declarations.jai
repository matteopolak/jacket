// Generated from docs/specs/stdlib/modules/windows.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file

Advapi32 :: #library,system "Advapi32"; @native(external)
Crt :: #library,system "ucrt"; @native(external)
DbgHelp :: #library,system "DbgHelp"; @native(external)
Dwmapi :: #library,system "Dwmapi"; @native(external)
Shlwapi :: #library,system "Shlwapi"; @native(external)
comdlg32 :: #library,system "comdlg32"; @native(external)
gdi :: #library,system "gdi32"; @native(external)
gl :: #library,system "opengl32"; @native(external)
kernel32 :: #library,system "kernel32"; @native(external)
ntdll :: #library,system "ntdll"; @native(external)
setupapi :: #library,system "setupapi"; @native(external)
shell32 :: #library,system "shell32"; @native(external)
user32 :: #library,system "user32"; @native(external)
winmm :: #library,system "winmm"; @native(external)
#scope_export

// Types and layouts

// Callback and opaque types

// Aliases and constants

// Native storage

// Native entry points

// Convenience operations

// Each platform's own declarations, parsed only when compiling for it.
#if OS == .WINDOWS {
    #load "declarations/windows.jai";
}
