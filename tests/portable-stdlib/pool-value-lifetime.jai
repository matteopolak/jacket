// Reviewed pool.md: field-based allocator guard, retained through bulk cleanup.
#import "Basic";
P :: #import "Pool";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

reject_setter :: (pool: *P.Pool) {
    selected := pool.block_allocator;
    previous := context.logger;
    before := errors;

    context.logger = capture;
    P.set_allocators(pool, .{ P.pool_allocator_proc, null });
    context.logger = previous;
    assert(errors == before + 1);
    assert(pool.block_allocator.proc == selected.proc && pool.block_allocator.data == selected.data);
}

returned_pool :: () -> P.Pool {
    pool: P.Pool;

    P.set_allocators(*pool);
    memory := cast(*u8) P.get(*pool, 32);

    assert(memory != null);
    memory[0] = 73;

    return pool;
}

main :: () {
    configured: P.Pool;

    P.set_allocators(*configured);
    reject_setter(*configured);

    for mode: 0..1 {
        free_on_reset := mode == 1;
        pool := returned_pool();

        assert(pool.current_pos[-32] == 73);
        pool.free_memblocks_on_reset = free_on_reset;
        reject_setter(*pool);
        P.reset(*pool);
        reject_setter(*pool);
        assert(pool.free_memblocks_on_reset == free_on_reset);
        assert(P.get(*pool, 32) != null);
        P.release(*pool);
        reject_setter(*pool);
        assert(pool.free_memblocks_on_reset == free_on_reset);
        assert(P.get(*pool, 32) != null);
        P.release(*pool);
    }

    // Repeated calls reuse the callee stack slot; ownership follows returned data.
    for 0..99 {
        pool := returned_pool();
        P.release(*pool);
    }

    print("ok\n");
}
