// test: only(.MACOS,.LINUX,.WINDOWS)
#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
#load "support/test-heap.jai";

live :: () -> s64 {
    report := D.make_leak_report(for_visualizer = true);

    defer D.deinit(*report);
    count: s64;

    for summary: report.sorted_summaries count += summary.count;

    return count;
}

// A heap made before anything else touched the debugger is still tracked, so destroying it forgets what it handed out.
main :: () {
    heap := D.create_heap(.{ heap_allocator, null });

    {
        push_allocator(heap);
        D.alloc(16);
        D.alloc(8);
    }

    assert(live() == 2);
    D.destroy_heap(heap);
    assert(live() == 0);
    assert(heap_destroyed);
    print("ok\n");
}
