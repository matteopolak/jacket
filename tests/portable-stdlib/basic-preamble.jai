B :: #import "Basic";
D :: #import "Basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
Pools :: #import "Pool";

calls: int;
nested_calls: int;
unexpected_logs: int;
bookkeeping_memory: *void;

capture :: (message: string, data: *void, info: Log_Info) {
    unexpected_logs += 1;
}

nested :: (allocator: Allocator) {
    D.Preamble();
    nested_calls += 1;
}

bookkeep :: (allocator: Allocator) {
    D.Preamble();
    B.assert(context.inside_memory_debugger);
    B.assert(context.logger == runtime_support_default_logger);
    nested(allocator);
    bookkeeping_memory = D.alloc(8);
    B.assert(bookkeeping_memory != null);
    calls += 1;
}

early :: (allocator: Allocator) {
    D.Preamble();
    B.assert(context.inside_memory_debugger);
    return;
}

flat_caps :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    B.assert(mode == .CAPS && old_memory != null && allocator_data == null);
    (cast(*string) old_memory).* = "modules/Flat_Pool";
    return cast(*void) cast(u64) Allocator_Caps.HINT_I_AM_A_FAST_BUMP_ALLOCATOR;
}

main :: () {
    push_context {
        context.logger = capture;
        B.assert(!context.inside_memory_debugger);
        bookkeep(context.allocator);
        #if OS == .WASM B.assert(calls == 0);
        else #if OS == BUILD_OS && CPU == BUILD_CPU B.assert(calls == 1);
        else B.assert(calls == 0);
        B.assert(nested_calls == 0 && !context.inside_memory_debugger && context.logger == capture);
        report := D.make_leak_report(.{ scan_global_memory_to_remove_leaks = false });

        B.assert(report.sorted_summaries.count == 0);
        D.deinit(*report);
        if bookkeeping_memory D.free(bookkeeping_memory);

        early(context.allocator);
        B.assert(!context.inside_memory_debugger && context.logger == capture);
        before := calls;

        bookkeep(B.temp);
        bookkeep(.{ Pools.pool_allocator_proc, null });
        bookkeep(.{ flat_caps, null });
        B.assert(calls == before && context.logger == capture);

        push_context {
            context.inside_memory_debugger = true;
            bookkeep(context.allocator);
            B.assert(context.inside_memory_debugger && context.logger == capture);
        }

        B.assert(calls == before);
        B.assert(unexpected_logs == 0);
    }

    B.print("ok\n");
}
