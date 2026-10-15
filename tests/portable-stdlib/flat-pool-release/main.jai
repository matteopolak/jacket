// Reviewed fini releases the reservation; real native map checks, no mocks.
#import "Basic";
F :: #import "Flat_Pool";

Libc :: #system_library "libc";
mprotect :: (address: *void, size: u64, protection: s32) -> s32 #foreign Libc;

main :: () {
    pool: F.Flat_Pool;

    F.init(*pool, 65536);
    base := pool.memory_base;
    if base == null exit(2);

    data := cast(*u8) F.get(*pool, 1);
    if data == null exit(3);

    data[0] = 77;
    if mprotect(base, 1, 3) != 0 exit(4);

    F.fini(*pool);
    if pool.memory_base != null || pool.address_limit != null || pool.current_point != null || pool.first_uncommitted_page != null exit(5);
    if mprotect(base, 1, 3) == 0 exit(6);

    F.fini(*pool);
    print("ok\n");
}
