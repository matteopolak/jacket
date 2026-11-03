#import "Basic";
P :: #import "Pool";

safe_query :: () {
    pool: P.Pool;

    defer P.release(*pool);
    memory := P.get(*pool, 100000);

    P.reset(*pool);
    owned := P.pool_allocator_proc(.IS_THIS_YOURS, 0, 0, memory, *pool);

    assert(owned == null);
}

owns :: (pool: *P.Pool, memory: *void) -> bool {
    return P.pool_allocator_proc(.IS_THIS_YOURS, 0, 0, memory, pool) != null;
}

safe_wrapped_query :: () {
    pool: P.Pool;

    defer P.release(*pool);
    memory := P.get(*pool, 100000);

    P.reset(*pool);
    assert(!owns(*pool, memory));
}

bad_resize :: () {
    pool: P.Pool;

    defer P.release(*pool);
    memory := New(int);

    free(memory);
    result := P.pool_allocator_proc(.RESIZE, size_of(int), size_of(int), memory, *pool);

    assert(result != null);
}

owns_and_reads :: (pool: *P.Pool, memory: *u8) -> bool {
    owned := owns(pool, memory);
    return owned || memory.* != 0;
}

bad_query_read :: () -> bool {
    pool: P.Pool;

    defer P.release(*pool);
    memory := cast(*u8) P.get(*pool, 100000);

    P.reset(*pool);

    return owns_and_reads(*pool, memory);
}
