// The C library calls the runtime, the allocators and the file backends make directly, declared once.
// They have no dependency on Basic, so the runtime can use them before any context exists.
#scope_file
libc :: #system_library "libc";

#scope_export
c_malloc :: (size: u64) -> *void #foreign libc "malloc";
c_realloc :: (memory: *void, size: u64) -> *void #foreign libc "realloc";
c_free :: (memory: *void) #foreign libc "free";
c_memmove :: (destination: *void, source: *void, count: u64) -> *void #foreign libc "memmove";
c_abort :: () #foreign libc "abort";
