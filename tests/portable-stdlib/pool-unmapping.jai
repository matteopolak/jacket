// Reviewed pool.md allocator selection/reset/ownership contracts.
#import "Basic";
P :: #import "Pool"(USE_UNMAPPING_ALLOCATOR = true);
U :: #import "Unmapping_Allocator";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

main :: () {
    pool: P.Pool;

    P.set_allocators(*pool);
    selected := pool.block_allocator;

    assert(selected.proc == U.unmapping_allocator_proc && selected.data != null);
    assert(pool.free_memblocks_on_reset);
    wrapper := cast(*U.Unmapping_Allocator) selected.data;
    ordinary := cast(*u8) P.get(*pool, 32);
    oversized := cast(*u8) P.get(*pool, 7000);

    assert(ordinary != null && oversized != null && wrapper.table.count == 2);
    ordinary[0] = 19;
    oversized[0] = 73;
    P.reset(*pool);
    assert(wrapper.table.count == 0 && pool.used_memblocks == null && pool.unused_memblocks == null);
    assert(ordinary[0] == 19 && oversized[0] == 73);
    previous := context.logger;

    context.logger = capture;
    P.set_allocators(*pool, context.allocator);
    context.logger = previous;
    assert(errors == 1 && pool.block_allocator.proc == selected.proc && pool.block_allocator.data == selected.data);
    assert(pool.free_memblocks_on_reset);
    fresh := P.get(*pool, 32);

    assert(fresh != null && fresh != ordinary && wrapper.table.count == 1);
    P.release(*pool);
    assert(wrapper.table.count == 0);
    // Pool releases backing allocations, not the selected allocator metadata.
    again := selected.proc(.ALLOCATE, 17, 0, null, selected.data);

    assert(again != null && wrapper.table.count == 1);
    selected.proc(.FREE, 0, 17, again, selected.data);
    U.deinit_unmapping_allocator(selected);

    explicit: P.Pool;

    P.set_allocators(*explicit, context.allocator);
    assert(explicit.block_allocator.proc == context.allocator.proc && explicit.block_allocator.data == context.allocator.data);
    assert(!explicit.free_memblocks_on_reset);
    scoped := context;
    scoped.allocator = .{ P.pool_allocator_proc, *explicit };
    // Selecting the backing allocator precedes using this pool as the context.
    push_context scoped {
        allocated := context.allocator.proc(.ALLOCATE, 32, 0, null, context.allocator.data);
        assert(allocated != null);
    }

    P.reset(*explicit);
    assert(explicit.unused_memblocks != null);
    P.release(*explicit);
    explicit.free_memblocks_on_reset = true;
    context.logger = capture;
    P.set_allocators(*explicit, context.allocator);
    context.logger = previous;
    assert(errors == 2 && explicit.free_memblocks_on_reset);
    print("ok\n");
}
