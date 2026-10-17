// Default public Pool aggregate values without allocating a block.
#import "Basic";
Pools :: #import "Pool";

main :: () {
    pool: Pools.Pool;

    assert(pool.memblock_size == Pools.POOL_BUCKET_SIZE_DEFAULT);
    assert(pool.oversized_size == Pools.POOL_OVERSIZED_SIZE_DEFAULT);
    assert(pool.alignment == 8);
    assert(!pool.overwrite_memory && !pool.free_memblocks_on_reset);
    assert(pool.current_memblock == null && pool.current_pos == null);
    assert(pool.bytes_left == 0);
    print("ok\n");
}
