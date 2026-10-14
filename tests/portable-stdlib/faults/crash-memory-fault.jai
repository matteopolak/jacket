#import "Basic";
C :: #import "Runtime_Support_Crash_Handler";
P :: #import "POSIX";

libc :: #system_library "libc";
native_memset :: (memory: *void, value: s32, count: u64) -> *void #foreign libc "memset";

main :: () {
    C.init();
    memory := P.mmap(null, cast(u64) P.sysconf(P._SC_PAGESIZE), P.PROT_NONE, P.MAP_PRIVATE | P.MAP_ANON, -1, 0);

    assert(memory != P.MAP_FAILED);
    native_memset(memory, 1, 1);
    assert(false, "protected native memory unexpectedly writable");
}
