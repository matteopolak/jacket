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
