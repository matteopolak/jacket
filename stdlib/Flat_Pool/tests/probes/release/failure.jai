// Safety policy for actual munmap failure: diagnose and retain retryable owner.
#import "Basic";
F :: #import "Flat_Pool";

main :: () {
    pool: F.Flat_Pool;

    F.init(*pool, 65536);
    base := pool.memory_base;
    data := cast(*u8) F.get(*pool, 1);

    if base == null || data == null exit(2);

    data[0] = 91;
    // An unaligned base is rejected by the real OS without releasing the map.
    pool.memory_base += 1;
    before := pool;

    F.fini(*pool);
    if pool.memory_base != before.memory_base || pool.current_point != before.current_point ||
       pool.first_uncommitted_page != before.first_uncommitted_page || pool.address_limit != before.address_limit exit(3);
    if data[0] != 91 exit(4);

    pool.memory_base = base;
    F.fini(*pool);
    if pool.memory_base != null exit(5);

    print("ok\n");
}
