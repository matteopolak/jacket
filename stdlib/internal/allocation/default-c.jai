allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .ALLOCATE {
        return obtain_contents(size, PREFIX);
    }

    if mode == .RESIZE {
        if !old_memory return obtain_contents(size, PREFIX);
        return regrow_contents(old_memory, size, PREFIX);
    }

    if mode == .FREE {
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

// The C allocator has no worker-owned inherited lock to reset after fork.
unsafe_reset :: inline () {}

#scope_file
#import "Basic";

// Room for the word aligned-blocks.jai keeps before the contents, kept 16 so they stay malloc-aligned.
PREFIX :: 16;
