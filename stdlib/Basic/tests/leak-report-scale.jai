// test: only(.MACOS,.LINUX,.WINDOWS)
#import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

options := D.Leak_Report_Options.{ scan_global_memory_to_remove_leaks = false };

SITES :: 600;
LINKS :: 3000;

// Recurses `depth` calls before allocating twice, so every depth is its own call stack and its own allocation site.
site :: (depth: s64, size: s64, kept: *[SITES * 2] *void, used: *s64) {
    if depth > 1 {
        site(depth - 1, size, kept, used);
        return;
    }

    for 0..1 {
        memory := D.alloc(size);

        memset(memory, 0, size);
        kept.*[used.*] = memory;
        used.* += 1;
    }
}

// The size grows with the depth so the summaries differ in weight; each counts two allocations.
many_sites :: () {
    kept: [SITES * 2] *void;
    used: s64;

    for depth: 1..SITES site(depth, 8 + depth, *kept, *used);

    report := D.make_leak_report(options);
    total: s64;
    ordered := true;

    assert(report.sorted_summaries.count == SITES);

    for summary, index: report.sorted_summaries {
        total += summary.count;
        assert(summary.count == 2 && summary.child_count == 0);
        if index > 0 && report.sorted_summaries[index - 1].bytes < summary.bytes ordered = false;
    }

    // Heaviest first.
    assert(ordered && total == SITES * 2);
    D.deinit(*report);
    for i: 0..used - 1 D.free(kept[i]);
}

// Each allocation points at the next and the last points back at the first, so the first owns all the others.
long_chain :: () {
    links := cast(**void) D.alloc(LINKS * size_of(*void));

    for i: 0..LINKS - 1 {
        link := cast(**void) D.alloc(size_of(*void));

        link.* = null;
        links[i] = link;
    }

    for i: 0..LINKS - 1 {
        next := links[(i + 1) % LINKS];
        (cast(**void) links[i]).* = next;
    }

    // The array keeps the addresses to free them; it does not own the chain.
    D.this_allocation_does_not_propagate_leak_ownership(links);

    report := D.make_leak_report(options);

    // The array, and the chain's head.
    assert(report.sorted_summaries.count == 2);

    chain_count: s64;
    chain_bytes: s64;

    for summary: report.sorted_summaries {
        chain_count += summary.child_count;
        chain_bytes += summary.child_bytes;
    }

    // The closing pointer would make a cycle and is not followed.
    assert(chain_count == LINKS - 1 && chain_bytes == (LINKS - 1) * size_of(*void));
    D.deinit(*report);

    for i: 0..LINKS - 1 D.free(links[i]);

    D.free(links);
}

// Marking an allocation finds its record by address.
many_marks :: () {
    blocks := cast(**void) D.alloc(LINKS * size_of(*void));

    for i: 0..LINKS - 1 {
        blocks[i] = D.alloc(16);
        memset(blocks[i], 0, 16);
    }

    D.this_allocation_does_not_propagate_leak_ownership(blocks);
    for i: 0..LINKS - 1 D.this_allocation_is_not_a_leak(blocks[i]);

    report := D.make_leak_report(options);

    assert(report.marked_as_non_leak_allocations == LINKS);
    assert(report.marked_as_non_leak_bytes == LINKS * 16);
    D.deinit(*report);

    for i: 0..LINKS - 1 D.free(blocks[i]);

    D.free(blocks);
}

main :: () {
    many_sites();
    long_chain();
    many_marks();
    print("ok\n");
}
