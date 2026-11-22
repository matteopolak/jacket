// Generated from docs/specs/stdlib/modules/macos.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "POSIX";
#import "Objective_C";

Boolean :: u8;
UTF32Char :: u32;
UTF16Char :: u16;
StringPtr :: *u8;
ConstStringPtr :: *u8;
ConstStr255Param :: *u8;
LangCode :: s16;
RegionCode :: s16;
ScriptCode :: s16;
Block :: *Objective_C_Block_Basic;
CoreServices :: #library,system "CoreServices"; @native(external)
corefoundation :: #library,system "corefoundation"; @native(external)
libc :: #library,system "c"; @native(external)
#scope_export

// Types and layouts

// Callback and opaque types

// Aliases and constants

// Native storage

// Native entry points

// Convenience operations

// Each platform's own declarations, parsed only when compiling for it.
#if OS == .MACOS {
    #load "declarations/macos.jai";
}
