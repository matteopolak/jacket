// The allocator procedure, backed by the C runtime's `malloc`. It handles `.ALLOCATE`, `.RESIZE` and `.FREE`, and returns null when memory runs out.
//
// ```jai
// memory := cast(*u8) allocator_proc(.ALLOCATE, 8, 0, null, null);
// memory[0] = 1;
// allocator_proc(.FREE, 0, 8, memory, null);
// ```
allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .ALLOCATE {
        return tagged(obtain_contents(size, PREFIX));
    }

    if mode == .RESIZE {
        if !old_memory return tagged(obtain_contents(size, PREFIX));
        if !is_tagged(old_memory) return refuse_foreign_pointer();
        return tagged(regrow_contents(old_memory, size, PREFIX));
    }

    if mode == .FREE {
        if !old_memory return null;
        if !is_tagged(old_memory) return refuse_foreign_pointer();

        untag(old_memory);
        release_contents(old_memory);

        return null;
    }

    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Default_Allocator C fallback";
        }

        return cast(*void) cast(u64) (Allocator_Caps.MULTIPLE_THREADS | .FREE | .ACTUALLY_RESIZE | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR);
    }

    if mode == .STARTUP || mode == .SHUTDOWN || mode == .THREAD_START || mode == .THREAD_STOP return null;

    assert(false, "unsupported C fallback allocator mode");

    return null;
}

// Does nothing here; the C allocator has no lock to reset after a `fork`.
unsafe_reset :: inline () {}

#scope_file
#import "Basic";

// Room for the two words kept before the contents, so they stay malloc-aligned: the tag, then the word aligned-blocks.jai keeps.
PREFIX :: 16;

// Every block this allocator hands out carries a tag in the word 16 bytes before its contents:
// this value mixed with the contents' address. A pointer that never came from here (a local, a
// global, memory from another allocator) or was already freed does not carry it, so a free or
// resize of one is refused before it reaches `free` and corrupts the heap. The word is inside the
// block's own prefix, and the region word next to it is read for every block anyway.
TAG :: 0x4a41434b45544231;

tag_slot :: (contents: *void) -> *u64 {
    return cast(*u64) (cast(*u8) contents - 16);
}

tagged :: (contents: *void) -> *void {
    if contents tag_slot(contents).* = TAG ^ cast(u64) contents;

    return contents;
}

is_tagged :: (contents: *void) -> bool {
    return tag_slot(contents).* == TAG ^ cast(u64) contents;
}

untag :: (contents: *void) {
    tag_slot(contents).* = 0;
}

// The caller handed over something that is not a live block. Nothing is released; it gets the report and null.
refuse_foreign_pointer :: () -> *void {
    if ENABLE_ASSERTS assert(false, "Default_Allocator received an invalid pointer or source extent");

    return null;
}
