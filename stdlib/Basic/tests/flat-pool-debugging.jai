// test: only(.MACOS,.LINUX,.WINDOWS)
#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
F :: #import "Flat_Pool";

live :: () -> count: s64, bytes: s64 {
    report := D.make_leak_report(for_visualizer = true);

    defer D.deinit(*report);
    count, bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count;
        bytes += summary.bytes;
    }

    return count, bytes;
}

main :: () {
    pool: F.Flat_Pool;
    other: F.Flat_Pool;

    defer F.fini(*other);

    F.init(*pool, 1024 * 1024);
    F.init(*other, 1024 * 1024);

    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *other });
        assert(D.alloc(5) != null);
    }

    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });
        D.alloc(10);
        D.alloc(20);
        assert(D.copy_string("abc") == "abc");
    }

    count, bytes := live();

    assert(count == 4 && bytes == 38);

    // Reset drops what the pool handed out and nothing else.
    F.reset(*pool);
    count, bytes = live();
    assert(count == 1 && bytes == 5);

    // The same addresses come back and are tracked as new allocations.
    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });
        assert(D.alloc(7) != null);
        assert(D.alloc(9) != null);
    }

    count, bytes = live();
    assert(count == 3 && bytes == 21);

    // A second reset, with nothing new in between, is harmless.
    F.reset(*pool);
    F.reset(*pool);
    count, bytes = live();
    assert(count == 1 && bytes == 5);

    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });
        D.alloc(3);
    }

    F.fini(*pool);
    count, bytes = live();
    assert(count == 1 && bytes == 5);

    // A pool that was finalized can be set up again and tracked afresh.
    F.init(*pool, 1024 * 1024);

    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });
        D.alloc(11);
    }

    count, bytes = live();
    assert(count == 2 && bytes == 16);
    F.fini(*pool);
    count, bytes = live();
    assert(count == 1 && bytes == 5);
    print("ok\n");
}
