// Pool with USE_UNMAPPING_ALLOCATOR: blocks come from a shared guard-page allocator
// (nothing to free by hand), recycling still works, and release hands the blocks back.
// Wasm has no virtual-memory calls, so the test is skipped there.
#import "Basic";
#import "Pool"(USE_UNMAPPING_ALLOCATOR = true);

main :: () {
    p: Pool;
    first := cast(*u8) get(*p, 100);
    assert(first != null);
    assert(p.block_allocator.proc != null);
    assert(p.block_allocator.proc != context.allocator.proc, "blocks do not come from the context allocator");
    assert((get_capabilities(p.block_allocator) & .HINT_I_AM_A_DEBUG_ALLOCATOR) != 0);
    for 0..99 first[it] = cast,trunc(u8) it;

    // Fill past one block, and take an oversized request.
    for 0..2000 {
        q := cast(*u8) get(*p, 64);
        assert(q != null && cast(u64) q % 8 == 0);
        memset(q, 0x5a, 64);
    }
    big := cast(*u8) get(*p, 100000 + 3);
    assert(big != null && cast(u64) big % 8 == 0);
    memset(big, 1, 100003);
    for 0..99 assert(first[it] == cast,trunc(u8) it, "earlier data survived");
    used, outband := calculate_used_memory(p);
    assert(used >= 2000 * 64 && outband == 1);

    // Reset recycles the blocks; release returns them to the guard allocator.
    reset(*p);
    again := get(*p, 16);
    assert(again != null);
    release(*p);
    assert(p.block_allocator.proc != null, "the pool keeps its block allocator");

    // A released pool can be used again; it keeps using the shared guard allocator.
    q := get(*p, 8);
    assert(q != null);
    other: Pool;
    get(*other, 8);
    assert(other.block_allocator.data == p.block_allocator.data, "pools share one guard allocator");
    release(*other);
    release(*p);

    // A caller-chosen block allocator is left alone.
    r: Pool;
    set_allocators(*r, context.allocator);
    get(*r, 8);
    assert(r.block_allocator.proc == context.allocator.proc);
    release(*r);
    print("ok\n");
}
