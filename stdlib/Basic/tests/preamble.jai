B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
Pools :: #import "Pool";
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS Flat_Pools :: #import "Flat_Pool";

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

        #if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
            caps, name := B.get_capabilities(.{ Flat_Pools.flat_pool_allocator_proc, null });
            B.assert((caps & .HINT_I_AM_A_FAST_BUMP_ALLOCATOR) != 0 && name == Flat_Pools.CAPS_VERSION_STRING);
            bookkeep(.{ Flat_Pools.flat_pool_allocator_proc, null });
        }

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
