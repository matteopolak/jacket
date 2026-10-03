// Canonical bootstrap allocation declarations.
Allocator_Mode :: enum {
    ALLOCATE :: 0;
    RESIZE :: 1;
    FREE :: 2;
    STARTUP :: 3;
    SHUTDOWN :: 4;
    THREAD_START :: 5;
    THREAD_STOP :: 6;
    CREATE_HEAP :: 7;
    DESTROY_HEAP :: 8;
    IS_THIS_YOURS :: 9;
    CAPS :: 10;
}

Allocator_Caps :: enum_flags u32 {
    MULTIPLE_THREADS :: 0x1;
    CREATE_HEAP :: 0x2;
    FREE :: 0x4;
    ACTUALLY_RESIZE :: 0x8;
    IS_THIS_YOURS :: 0x10;

    HINT_I_AM_A_FAST_BUMP_ALLOCATOR :: 0x0100_0000;
    HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR :: 0x0200_0000;
    HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE :: 0x0400_0000;
    HINT_I_AM_A_DEBUG_ALLOCATOR :: 0x0800_0000;
}

Allocator_Proc :: #type (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void;

Allocator :: struct {
    proc: Allocator_Proc;
    data: *void;
}
