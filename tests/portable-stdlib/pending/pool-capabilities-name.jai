#import "Basic";
Pools :: #import "Pool";

main :: () {
    flags, name := get_capabilities(.{ Pools.pool_allocator_proc, null });
    assert((flags & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) != 0 && name.count > 0);
    print("ok\n");
}
