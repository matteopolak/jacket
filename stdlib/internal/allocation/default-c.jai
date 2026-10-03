allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .ALLOCATE {
        if size < 0 return null;
        return c_malloc(cast(u64) size);
    }

    if mode == .RESIZE {
        if size < 0 return null;
        return c_realloc(old_memory, cast(u64) size);
    }

    if mode == .FREE {
        c_free(old_memory);
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

libc :: #system_library "libc";
c_malloc :: (size: u64) -> *void #foreign libc "malloc";
c_realloc :: (memory: *void, size: u64) -> *void #foreign libc "realloc";
c_free :: (memory: *void) #foreign libc "free";
