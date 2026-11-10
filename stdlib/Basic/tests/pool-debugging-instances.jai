// Different parameters make separate debuggers, each with its own records.
A :: #import "internal/basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
B :: #import "internal/basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false, TEMP_ALLOCATOR_OVERWRITE_MEMORY = true);
#import "Basic";
P :: #import "Pool";

live_a :: () -> s64 {
    report := A.make_leak_report(for_visualizer = true);

    defer A.deinit(*report);
    count: s64;

    for summary: report.sorted_summaries count += summary.count;

    return count;
}

live_b :: () -> s64 {
    report := B.make_leak_report(for_visualizer = true);

    defer B.deinit(*report);
    count: s64;

    for summary: report.sorted_summaries count += summary.count;

    return count;
}

// Two debuggers that both track allocations each see a pool's blocks, resets and release, whichever started first.
main :: () {
    // A debugger starts with its first allocation and only hears of what happens after that.
    A.free(A.alloc(1));
    B.free(B.alloc(1));

    pool: P.Pool;

    P.set_allocators(*pool, context.allocator);
    base_a := live_a();
    base_b := live_b();

    {
        push_allocator(Allocator.{ P.pool_allocator_proc, *pool });
        A.alloc(10);
        B.alloc(20);
    }

    // Each debugger sees the range it made and the pool's block.
    assert(live_a() == base_a + 2);
    assert(live_b() == base_b + 2);

    P.reset(*pool);
    assert(live_a() == base_a + 1);
    assert(live_b() == base_b + 1);

    P.release(*pool);
    assert(live_a() == base_a);
    assert(live_b() == base_b);
    print("ok\n");
}
