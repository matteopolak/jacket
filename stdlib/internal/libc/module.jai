// The C library calls the runtime, the allocators and the file backends make directly, declared once.
// They have no dependency on Basic, so the runtime can use them before any context exists.
//
// The wrappers take `u64` sizes and pass C's `size_t`, which is 4 bytes on a 32-bit target: a
// 64-bit argument after a pointer would land in the wrong register on ARM.
#scope_file
libc :: #system_library "libc"; @native(external)

#if size_of(*void) == 4 {
    C_Size :: u32;
} else {
    C_Size :: u64;
}

malloc :: (size: C_Size) -> *void #foreign libc;
realloc :: (memory: *void, size: C_Size) -> *void #foreign libc;
memmove :: (destination: *void, source: *void, count: C_Size) -> *void #foreign libc;

#if OS != .WINDOWS {
    #if size_of(*void) == 4 {
        C_Signed_Size :: s32;
    } else {
        C_Signed_Size :: s64;
    }

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

#if OS != .WINDOWS {
    // Write up to `count` bytes of `data` to `fd`; the count written, or -1.
    c_write :: inline (fd: s32, data: *u8, count: u64) -> s64 #no_context {
        return write(fd, data, cast,trunc(C_Size) count);
    }
}
