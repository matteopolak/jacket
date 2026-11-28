// What an [Allocator] procedure is asked to do. `size`, `old_size` and `old_memory` carry the
// request; modes an allocator doesn't support return null.
Allocator_Mode :: enum {
    ALLOCATE :: 0; // Returns `size` new bytes.
    RESIZE :: 1; // Returns `size` bytes holding the first `old_size` bytes of `old_memory`, which may move.
    FREE :: 2; // Releases `old_memory`.
    STARTUP :: 3;
    SHUTDOWN :: 4;
    THREAD_START :: 5;
    THREAD_STOP :: 6;
    CREATE_HEAP :: 7; // Returns a new heap to pass as `allocator_data`.
    DESTROY_HEAP :: 8; // Releases the heap in `old_memory` and everything allocated from it.
    IS_THIS_YOURS :: 9; // Returns non-null when `old_memory` came from this allocator.
    CAPS :: 10; // Returns the allocator's [Allocator_Caps], cast to a pointer.
}

// What an allocator reports for [Allocator_Mode.CAPS]. The low bits say which modes it supports;
// the `HINT_` bits say what kind of allocator it is.
//
// `old_memory` is null or points to a `string` the allocator sets to its name, which it owns.
// Check for null before writing it: the memory debugger asks with null.
Allocator_Caps :: enum_flags u32 {
    MULTIPLE_THREADS :: 0x1; // Safe to call from several threads at once.
    CREATE_HEAP :: 0x2; // Supports `.CREATE_HEAP` and `.DESTROY_HEAP`.
    FREE :: 0x4; // `.FREE` releases memory; without it, freeing does nothing.
    ACTUALLY_RESIZE :: 0x8; // `.RESIZE` can grow in place instead of always copying.
    IS_THIS_YOURS :: 0x10; // Answers `.IS_THIS_YOURS`.

    HINT_I_AM_A_FAST_BUMP_ALLOCATOR :: 0x0100_0000; // A pool or arena: cheap allocation, frees all at once.
    HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR :: 0x0200_0000;
    HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE :: 0x0400_0000; // Temporary storage, reset with `reset_temporary_storage`.
    HINT_I_AM_A_DEBUG_ALLOCATOR :: 0x0800_0000; // Traps misuse at a cost in speed or memory.
}

// The procedure behind an [Allocator]. `allocator_data` is the allocator's own state, such as a
// pool or a heap.
Allocator_Proc :: #type (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void;

// A procedure and the state it works on. `context.allocator` is the one `alloc`, `free` and
// `New` use when none is passed.
Allocator :: struct {
    proc: Allocator_Proc;
    data: *void; // Passed as `allocator_data` on every call.
}
