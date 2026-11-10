#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
P :: #import "Pool";

live :: () -> s64 {
    report := D.make_leak_report(for_visualizer = true);

    defer D.deinit(*report);
    count: s64;

    for summary: report.sorted_summaries count += summary.count;

    return count;
}

main :: () {
    pool: P.Pool;

    P.set_allocators(*pool, context.allocator);
    base := live();

    {
        push_allocator(Allocator.{ P.pool_allocator_proc, *pool });
        D.alloc(10);
        D.alloc(20);
        assert(D.copy_string("abc") == "abc");
    }

    // The three ranges it handed out, and the block they sit in.
    assert(live() == base + 4);

    // Reset forgets the ranges and keeps the block for reuse.
    P.reset(*pool);
    assert(live() == base + 1);

    {
        push_allocator(Allocator.{ P.pool_allocator_proc, *pool });
        assert(D.alloc(7) != null);
    }

    assert(live() == base + 2);
    P.release(*pool);
    assert(live() == base);

    // An oversized request has a block of its own, and a pool that is never released leaks its blocks.
    leaked: P.Pool;

    P.set_allocators(*leaked, context.allocator);
    P.get(*leaked, 100000);
    P.get(*leaked, 8);
    assert(live() == base + 2);
    P.release(*leaked);
    assert(live() == base);
    print("ok\n");
}
