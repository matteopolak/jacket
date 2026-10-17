B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
HT :: #import "Hash_Table";

root: *void;
discarded: int;

side_effect :: () -> *void {
    discarded += 1;
    return *discarded;
}

totals :: (report: D.Leak_Report) -> (count: s64, bytes: s64) {
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count;
        bytes += summary.bytes;
    }

    return count, bytes;
}

main :: () {
    B.this_allocation_is_not_a_leak(side_effect());
    B.this_allocation_leaks_todo(side_effect());
    B.this_allocation_does_not_propagate_leak_ownership(side_effect());
    B.assert(discarded == 0);
    early := D.alloc(8);

    memset(early, 0, 8);
    D.allocations_before_this_call_will_not_be_considered_leaks();
    late := D.alloc(16);

    memset(late, 0, 16);
    exempt := D.alloc(24);

    memset(exempt, 0, 24);
    D.this_allocation_is_not_a_leak(exempt);
    D.this_allocation_is_not_a_leak(null);
    parent := D.alloc(size_of(*void));
    child := D.alloc(32);

    memset(child, 0, 32);
    (cast(**void) parent).* = child;
    root = parent;
    report := D.make_leak_report();
    count, bytes := totals(report);

    B.assert(count == 1 && bytes == 16);
    B.assert(report.marked_as_non_leak_allocations == 4);
    B.assert(report.marked_as_non_leak_bytes == 8 + 24 + size_of(*void) + 32);
    D.deinit(*report);
    B.assert(report.sorted_summaries.count == 0 && report.leak_combiner_table.count == 0);
    visual := D.make_leak_report(for_visualizer = true);
    visual_count, visual_bytes := totals(visual);

    B.assert(visual_count == 5 && visual_bytes == 8 + 16 + 24 + size_of(*void) + 32);
    D.deinit(*visual);

    report = D.make_leak_report(.{ scan_global_memory_to_remove_leaks = false, scan_leaks_to_combine = false });
    count, bytes = totals(report);
    B.assert(count == 3 && bytes == 16 + size_of(*void) + 32);
    for i: 0..99 HT.table_add(*report.leak_combiner_table, cast(*void) (cast(u64) 0x10000 + cast(u64) i), .{});

    B.assert(report.leak_combiner_table.count == 105);
    D.deinit(*report);
    D.this_allocation_leaks_todo(late);
    D.this_allocation_does_not_propagate_leak_ownership(null);
    D.this_allocation_does_not_propagate_leak_ownership(*report);
    D.free(early);
    D.free(late);
    D.free(exempt);
    D.free(parent);
    D.free(child);
    root = null;
    report = D.make_leak_report();
    count, bytes = totals(report);
    B.assert(count == 0 && bytes == 0);
    D.deinit(*report);
    B.print("ok\n");
}
