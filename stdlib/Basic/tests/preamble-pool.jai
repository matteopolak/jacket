// test: only(.MACOS,.LINUX,.WINDOWS)
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

// What a custom allocator proc does first.
custom :: (allocator: Allocator) {
    D.Preamble();
}

// A custom allocator under a pool starts the debugger like any other, so the pool's first block is seen even though
// nothing has gone through `alloc` yet.
main :: () {
    pool: P.Pool;

    P.set_allocators(*pool, context.allocator);
    custom(.{ P.pool_allocator_proc, *pool });
    P.get(*pool, 16);
    assert(live() == 1);
    P.release(*pool);
    assert(live() == 0);
    print("ok\n");
}
