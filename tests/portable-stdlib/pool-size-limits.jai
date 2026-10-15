// Independent allocation-safety regression; nonnegative sizes must never wrap.
#import "Basic";
P :: #import "Pool";

Tracking :: struct {
    base: Allocator;
    requests: s64;
    live: s64;
    byte_limit: s64 = 100000;
}

tracked :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;

    if mode == .ALLOCATE {
        state.requests += 1;
        assert(requested_size >= 0, "Pool forwarded an overflowed allocation size");
        if requested_size > state.byte_limit return null;

        result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);
        if result state.live += 1;

        return result;
    }

    if mode == .FREE state.live -= 1;

    return state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);
}

main :: () {
    tracking := Tracking.{ base = context.allocator };
    pool: P.Pool;

    P.set_allocators(*pool, .{ tracked, *tracking });
    assert(P.get(*pool, 0x7fffffffffffffff) == null);
    assert(tracking.requests == 0 && tracking.live == 0);
    assert(pool.current_memblock == null && pool.out_of_band_allocations == null);

    pool.memblock_size = 0x7fffffffffffffff;
    assert(P.get(*pool, 1) == null);
    assert(tracking.requests == 0 && pool.used_memblocks == null);
    pool.memblock_size = P.POOL_BUCKET_SIZE_DEFAULT;

    ordinary := cast(*u8) P.get(*pool, 32);

    assert(ordinary != null && tracking.live == 1);
    ordinary[0] = 73;
    old_position := pool.current_pos;
    old_left := pool.bytes_left;

    tracking.byte_limit = 0;
    assert(P.get(*pool, 7000) == null);
    assert(ordinary[0] == 73 && pool.current_pos == old_position && pool.bytes_left == old_left);
    assert(pool.out_of_band_allocations == null && tracking.live == 1);
    P.release(*pool);
    assert(tracking.live == 0);
    print("ok\n");
}
