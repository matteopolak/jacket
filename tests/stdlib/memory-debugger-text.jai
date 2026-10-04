Debugger :: #import "Basic"(MEMORY_DEBUGGER=true, VISUALIZE_MEMORY_DEBUGGER=false);

main :: () -> s64 {
    memory := Debugger.alloc(19);
    if !memory return 1;
    options: Debugger.Leak_Report_Options;
    options.scan_global_memory_to_remove_leaks = false;
    options.scan_leaks_to_combine = false;
    report: Debugger.Leak_Report = Debugger.make_leak_report(options);
    found := false;
    for summary: report.sorted_summaries if summary.bytes == 19 found = true;
    Debugger.log_leak_report(report);
    Debugger.deinit(*report);
    if !found return 3;
    Debugger.free(memory);
    Debugger.deinit_memory_debugger();
    return 0;
}
