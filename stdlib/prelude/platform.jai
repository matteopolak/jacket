// Canonical bootstrap platform declarations.
Operating_System_Tag :: enum u32 {
    NONE :: 0;
    KRAMPOS :: 1;
    WINDOWS :: 2;
    LINUX :: 3;
    ANDROID :: 4;
    IOS :: 5;
    MACOS :: 6;
    NN_SWITCH :: 7;
    PS4 :: 8;
    PS5 :: 9;
    XBOX :: 10;
    WASM :: 11;
    NN_SWITCH2 :: 12;
}

CPU_Tag :: enum u32 {
    UNINITIALIZED :: 0;
    KRAMPU :: 1;
    CUSTOM :: 2;
    X64 :: 3;
    ARM64 :: 4;
}

// The CPU family `CPU_Tag` cannot name: every CPU it lacks is `.CUSTOM`. `ARCHITECTURE` is the
// compiler's constant of this type.
Architecture :: enum u32 {
    X64 :: 0;
    ARM64 :: 1;
    X86 :: 2;
    ARM :: 3;
    S390X :: 4;
    POWERPC64 :: 5;
    WASM32 :: 6;
    WASM64 :: 7;
    AVR :: 8;
}

Workspace :: s64;

get_current_workspace :: () -> Workspace #compiler;

__reg :: #type,distinct u16;
