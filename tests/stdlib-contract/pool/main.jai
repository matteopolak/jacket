// Pool allocation alignment, accounting, and bulk reset/release behavior.
#import "Basic";
Pool :: #import "Pool";

main :: () {
    pool: Pool.Pool;

    assert(pool.memblock_size == 65536 && pool.oversized_size == 6554 && pool.alignment == 8);
    Pool.set_allocators(*pool, context.allocator);
    assert(pool.block_allocator.proc == context.allocator.proc);
    a := Pool.get(*pool, 24);

    assert(a != null && cast(u64) a % 8 == 0);
    inband, outband := Pool.calculate_used_memory(pool);

    assert(inband >= 24 && outband == 0);
    large := Pool.get(*pool, 6554);

    assert(large != null && cast(u64) large % 8 == 0);
    inband, outband = Pool.calculate_used_memory(pool);
    assert(outband == 1);
    Pool.reset(*pool);
    inband, outband = Pool.calculate_used_memory(pool);
    assert(inband == 0 && outband == 0);
    Pool.release(*pool);
    print("ok\n");
}
