B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

options := D.Leak_Report_Options.{ scan_global_memory_to_remove_leaks = false };

// A three-link chain: root -> middle -> last. `mark_root` and `mark_middle` are 0 (unmarked), 1 (not a leak)
// or 2 (does not propagate ownership).
Counts :: struct {
    leaks: s64;
    first_bytes: s64;
    first_children: s64;
    marked: s64;
    marked_bytes: s64;
}

chain :: (mark_root: int, mark_middle: int) -> Counts {
    root := D.alloc(16);
    middle := D.alloc(32);
    last := D.alloc(64);

    memset(root, 0, 16);
    memset(middle, 0, 32);
    memset(last, 0, 64);
    (cast(**void) root).* = middle;
    (cast(**void) middle).* = last;
    if mark_root == 1 D.this_allocation_is_not_a_leak(root);
    if mark_root == 2 D.this_allocation_does_not_propagate_leak_ownership(root);
    if mark_middle == 1 D.this_allocation_is_not_a_leak(middle);
    if mark_middle == 2 D.this_allocation_does_not_propagate_leak_ownership(middle);

    report := D.make_leak_report(options);
    result := Counts.{ leaks = report.sorted_summaries.count, marked = report.marked_as_non_leak_allocations, marked_bytes = report.marked_as_non_leak_bytes };

    if result.leaks > 0 {
        result.first_bytes = report.sorted_summaries[0].bytes;
        result.first_children = report.sorted_summaries[0].child_count;
    }

    D.deinit(*report);
    D.free(root);
    D.free(middle);
    D.free(last);

    return result;
}

main :: () {
    plain := chain(0, 0);

    B.assert(plain.leaks == 1 && plain.first_bytes == 16 && plain.first_children == 2 && plain.marked == 0);
    // What a non-leak reaches is hidden but not counted as marked.
    root_marked := chain(1, 0);

    B.assert(root_marked.leaks == 0 && root_marked.marked == 1 && root_marked.marked_bytes == 16);
    middle_marked := chain(0, 1);

    B.assert(middle_marked.leaks == 1 && middle_marked.first_bytes == 16 && middle_marked.first_children == 0);
    B.assert(middle_marked.marked == 1 && middle_marked.marked_bytes == 32);
    root_stops := chain(2, 0);

    B.assert(root_stops.leaks == 2 && root_stops.first_bytes == 32 && root_stops.first_children == 1);
    middle_stops := chain(0, 2);

    B.assert(middle_stops.leaks == 2 && middle_stops.first_bytes == 64 && middle_stops.marked == 0);
    // A child that is only reachable from a non-leak still belongs to another leaking owner.
    first := D.alloc(16);
    second := D.alloc(16);
    shared := D.alloc(48);

    memset(first, 0, 16);
    memset(second, 0, 16);
    memset(shared, 0, 48);
    (cast(**void) first).* = shared;
    (cast(**void) second).* = shared;
    D.this_allocation_is_not_a_leak(first);
    report := D.make_leak_report(options);

    B.assert(report.sorted_summaries.count == 1 && report.sorted_summaries[0].bytes == 16);
    B.assert(report.sorted_summaries[0].child_count == 1 && report.sorted_summaries[0].child_bytes == 48);
    B.assert(report.marked_as_non_leak_allocations == 1);
    D.deinit(*report);
    D.free(first);
    D.free(second);
    D.free(shared);
    B.print("ok\n");
}
