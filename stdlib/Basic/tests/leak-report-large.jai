// test: only(.MACOS,.LINUX,.WINDOWS)
#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
F :: #import "Flat_Pool";

root: *void;

COUNT :: 4000;
CHILD_SIZE :: 24;

children_of_owners :: (report: D.Leak_Report) -> count: s64, bytes: s64 {
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.child_count;
        bytes += summary.child_bytes;
    }

    return count, bytes;
}

leaks_in :: (report: D.Leak_Report) -> s64 {
    total: s64;
    for summary: report.sorted_summaries total += summary.count + summary.child_count;
    return total;
}

// A report over thousands of allocations: one holder points into each of them.
many :: () {
    children := cast(**u8) D.alloc(COUNT * size_of(*u8));
    holder := cast(**u8) D.alloc(COUNT * size_of(*u8));

    for i: 0..COUNT - 1 {
        child := cast(*u8) D.alloc(CHILD_SIZE);

        memset(child, 0, CHILD_SIZE);
        children[i] = child;

        // Pointers into the middle of a range and at its last byte still belong to it.
        holder[i] = child + ifx i % 2 == 0 then 9 else CHILD_SIZE - 1;
    }

    // The array only keeps the child addresses so they can be freed; it does not own them.
    D.this_allocation_does_not_propagate_leak_ownership(children);

    report := D.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes := children_of_owners(report);

    assert(count == COUNT && bytes == COUNT * CHILD_SIZE);
    assert(leaks_in(report) == COUNT + 2);
    D.deinit(*report);

    root = holder;
    report = D.make_leak_report();

    // Everything the global holder points into is reachable, and only the children array is left.
    assert(leaks_in(report) == 1);
    D.deinit(*report);
    root = null;

    for i: 0..COUNT - 1 D.free(children[i]);

    D.free(children);
    D.free(holder);
}

// A range inside a block is the closer owner; a pointer in the block but outside every range reaches the block.
inside_block :: () {
    pool: F.Flat_Pool;

    F.init(*pool, 1024 * 1024);
    defer F.fini(*pool);

    range: *u8;

    {
        push_allocator(Allocator.{ F.flat_pool_allocator_proc, *pool });
        range = cast(*u8) D.alloc(64);
    }

    memset(range, 0, 64);
    holder := cast(**u8) D.alloc(2 * size_of(*u8));

    holder[0] = range + 8;
    holder[1] = range + 64;

    report := D.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes := children_of_owners(report);

    // The range goes to the holder as a range, and the block as a block that is larger than the range.
    assert(count == 2 && bytes > 64 + 64);
    D.deinit(*report);

    root = holder;
    report = D.make_leak_report();
    assert(leaks_in(report) == 0);
    D.deinit(*report);
    root = null;
    D.free(holder);
}

main :: () {
    many();
    inside_block();
    print("ok\n");
}
