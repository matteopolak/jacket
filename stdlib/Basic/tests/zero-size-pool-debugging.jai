// test: only(.MACOS,.LINUX,.WINDOWS)
#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
F :: #import "Flat_Pool";
V :: #import "internal/allocation/virtual-memory";

exercise :: () {
    pool: F.Flat_Pool;

    defer F.fini(*pool);
    push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });

    first := D.copy_string("");
    second := D.copy_string("");
    text := D.copy_string("abc");

    assert(first.count == 0 && second.count == 0 && text == "abc");
    assert(first.data == text.data && second.data == text.data);

    empty := D.alloc(0);
    resized := D.realloc(empty, 8, 0);

    assert(empty == resized);
    memset(resized, 0, 8);

    report := D.make_leak_report(for_visualizer = true);
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count;
        bytes += summary.bytes;
    }

    // The two nonempty ranges and the page the pool committed for them.
    assert(count == 3 && bytes == 11 + cast(s64) V.page_size());
    D.deinit(*report);
}

#run exercise();

main :: () {
    exercise();
    print("ok\n");
}
