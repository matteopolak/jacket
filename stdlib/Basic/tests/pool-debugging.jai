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

    // The pool's blocks go straight to its block allocator, so only the three ranges it handed out show.
    assert(live() == base + 3);
    P.reset(*pool);
    assert(live() == base);

    {
        push_allocator(Allocator.{ P.pool_allocator_proc, *pool });
        assert(D.alloc(7) != null);
    }

    assert(live() == base + 1);
    P.release(*pool);
    assert(live() == base);
    print("ok\n");
}
