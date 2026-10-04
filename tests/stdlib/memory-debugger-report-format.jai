Debugger :: #import "Basic"(MEMORY_DEBUGGER=true, VISUALIZE_MEMORY_DEBUGGER=false);
#import "String";

// log_leak_report writes one block per leak (size, allocation count, stack trace with the
// procedure names padded), then the totals.

captured: [..] u8;

capture :: (message: string, data: *void, info: Log_Info) {
    for message Debugger.array_add(*captured, it);
}

leaky :: () -> *void {
    return Debugger.alloc(100);
}

main :: () -> s64 {
    memory := leaky();
    options: Debugger.Leak_Report_Options;
    options.scan_global_memory_to_remove_leaks = false;
    options.scan_leaks_to_combine = false;
    report := Debugger.make_leak_report(options);

    old := context.logger;
    context.logger = capture;
    Debugger.log_leak_report(report);
    context.logger = old;

    text: string;
    text.data = captured.data;
    text.count = captured.count;
    if !contains(text, "----- 100 bytes in 1 allocation -----") return 1;
    if !contains(text, "leaky ") || !contains(text, "memory-debugger-report-format.jai:") return 2;
    if !contains(text, "Total: 100 bytes in 1 allocation.") return 3;
    if !contains(text, "Marked as non-leaks: 0 bytes in 0 allocations.") return 4;

    Debugger.deinit(*report);
    Debugger.free(memory);
    Debugger.deinit_memory_debugger();
    return 0;
}
