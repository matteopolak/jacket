Basic :: #import "Basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
Node :: struct {
    child: *void;
}

totals :: (report: Basic.Leak_Report) -> (count: s64, bytes: s64) {
    count: s64;
    bytes: s64;
    for summary: report.sorted_summaries {
        count += summary.count + summary.child_count;
        bytes += summary.bytes + summary.child_bytes;
    }
    return count, bytes;
}
main :: () -> s64 {
    Basic.allocations_before_this_call_will_not_be_considered_leaks();
    options: Basic.Leak_Report_Options;
    options.scan_global_memory_to_remove_leaks = false;
    options.scan_leaks_to_combine = false;
    first := Basic.alloc(16);
    second := Basic.alloc(24);
    report := Basic.make_leak_report(options);
    count, bytes := totals(report);
    Basic.deinit(*report);
    if count != 2 || bytes != 40 return 1;
    Basic.this_allocation_is_not_a_leak(first);
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    if count != 1 || bytes != 24 || report.marked_as_non_leak_allocations != 1 || report.marked_as_non_leak_bytes != 16 return 2;
    Basic.deinit(*report);
    replacement := Basic.realloc(second, 48, 24);
    if !replacement return 3;
    second = replacement;
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    Basic.deinit(*report);
    if count != 1 || bytes != 48 return 4;
    Basic.free(first);
    Basic.free(second);
    parent := Basic.New(Node);
    child := Basic.alloc(32);
    parent.child = child;
    options.scan_leaks_to_combine = true;
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    if count != 2 || bytes != size_of(Node) + 32 return 5;
    roots: s64;
    children: s64;
    for summary: report.sorted_summaries {
        roots += summary.count;
        children += summary.child_count;
    }
    Basic.deinit(*report);
    if roots != 1 || children != 1 return 6;
    Basic.this_allocation_is_not_a_leak(parent);
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    Basic.deinit(*report);
    if count != 0 || bytes != 0 return 7;
    Basic.this_allocation_does_not_propagate_leak_ownership(parent);
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    Basic.deinit(*report);
    if count != 1 || bytes != 32 return 8;
    Basic.free(child);
    Basic.free(parent);
    report = Basic.make_leak_report(options);
    count, bytes = totals(report);
    Basic.deinit(*report);
    if count != 0 || bytes != 0 return 9;
    Basic.deinit_memory_debugger();
    return 0;
}
