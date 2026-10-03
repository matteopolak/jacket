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

Workspace :: s64;

get_current_workspace :: () -> Workspace #compiler;

__reg :: #type,distinct u16;
