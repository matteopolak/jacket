#import "Basic";
P :: #import "Pool";

owns :: (pool: *P.Pool, memory: *void) -> bool {
    return P.pool_allocator_proc(.IS_THIS_YOURS, 0, 0, memory, pool) != null;
}

main :: () {
    pool: P.Pool;

    defer P.release(*pool);

    small := P.get(*pool, 64);
    oversized := cast(*u8) P.get(*pool, 100000); // over the oversized threshold: a block of its own
    medium := cast(*u8) P.get(*pool, pool.oversized_size);

    assert(owns(*pool, small));
    assert(owns(*pool, oversized) && owns(*pool, oversized + 99999));
    assert(owns(*pool, medium));

    outside: s64;

    assert(!owns(*pool, *outside));

    P.reset(*pool);
    assert(!owns(*pool, oversized));

    print("ok\n");
}
