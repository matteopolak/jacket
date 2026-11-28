// The C library calls the runtime, the allocators and the file backends make directly, declared once.
// They have no dependency on Basic, so the runtime can use them before any context exists.
//
// The wrappers take `u64` sizes and pass C's `size_t`, which is 4 bytes on a 32-bit target: a
// 64-bit argument after a pointer would land in the wrong register on ARM.
// C's `size_t` and `ssize_t`, as wide as a pointer: what `#foreign` declarations of C calls
// take and return for sizes.
#if size_of(*void) == 4 {
    C_Size :: u32;
    C_Signed_Size :: s32;
} else {
    C_Size :: u64;
    C_Signed_Size :: s64;
}

#scope_file
libc :: #system_library "libc"; @native(external)

malloc :: (size: C_Size) -> *void #foreign libc;
realloc :: (memory: *void, size: C_Size) -> *void #foreign libc;
memmove :: (destination: *void, source: *void, count: C_Size) -> *void #foreign libc;

#if OS != .WINDOWS {
    read :: (fd: s32, data: *void, count: C_Size) -> C_Signed_Size #foreign libc;
    write :: (fd: s32, data: *void, count: C_Size) -> C_Signed_Size #foreign libc; @WritesOutput(0)
}

#scope_export
c_malloc :: inline (size: u64) -> *void #no_context {
    return malloc(cast,trunc(C_Size) size);
}

c_realloc :: inline (memory: *void, size: u64) -> *void #no_context {
    return realloc(memory, cast,trunc(C_Size) size);
}

c_free :: (memory: *void) #foreign libc "free"; @FreesArgument

c_memmove :: inline (destination: *void, source: *void, count: u64) -> *void #no_context {
    return memmove(destination, source, cast,trunc(C_Size) count);
}

c_abort :: () #foreign libc "abort"; @Exits

// Ends the process with `status` after the C library's exit handlers run.
c_exit :: (status: s32) #foreign libc "exit"; @Exits

// The text for an `errno` value, in storage the C library owns.
c_strerror :: (error: s32) -> *u8 #foreign libc "strerror";

// The value of the environment variable `name` (a zero-terminated string), or null when it is not set.
c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";

// The calling thread's `errno`; read or set it through the pointer.
#if OS == .MACOS {
    c_errno_address :: () -> *s32 #foreign libc "__error";
} else #if OS == .WINDOWS {
    c_errno_address :: () -> *s32 #foreign libc "_errno";
} else {
    c_errno_address :: () -> *s32 #foreign libc "__errno_location";
}

#if OS != .WINDOWS {
    // Read up to `count` bytes into `data` from `fd`; the count read, 0 at the end, or -1.
    c_read :: inline (fd: s32, data: *void, count: u64) -> s64 #no_context {
        return read(fd, data, cast,trunc(C_Size) count);
    }

    // Write up to `count` bytes of `data` to `fd`; the count written, or -1.
    c_write :: inline (fd: s32, data: *void, count: u64) -> s64 #no_context {
        return write(fd, data, cast,trunc(C_Size) count);
    }
}
