B :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

totals :: (report: B.Leak_Report) -> (count: s64, bytes: s64) {
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count;
        bytes += summary.bytes;
    }

    return count, bytes;
}

main :: () {
    B.allocations_before_this_call_will_not_be_considered_leaks();
    values: [..] int;

    B.array_reserve(*values, 4);
    report := B.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes := totals(report);

    B.assert(count == 1 && bytes == 4 * size_of(int));
    B.deinit(*report);
    B.array_add(*values, 1);
    B.array_reserve(*values, 9);
    report = B.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes = totals(report);
    B.assert(count == 1 && bytes == 9 * size_of(int) && values[0] == 1);
    B.deinit(*report);
    B.array_reset(*values);
    report = B.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes = totals(report);
    B.assert(count == 0 && bytes == 0);
    B.deinit(*report);
    raw := B.alloc(13);

    report = B.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes = totals(report);
    B.assert(count == 1 && bytes == 13 && raw != null);
    B.deinit(*report);
    B.free(raw);
    report = B.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });
    count, bytes = totals(report);
    B.assert(count == 0 && bytes == 0);
    B.deinit(*report);
}
