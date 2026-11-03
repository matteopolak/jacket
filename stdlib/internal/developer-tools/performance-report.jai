get_plugin :: () -> *Plugin {
    result := New(Plugin);
    result.allocator = context.allocator;
    result.before_intercept = request_measurements;
    result.message = collect_measurements;
    result.finish = write_measurements;
    result.shutdown = release_report;

    return result;
}

#scope_file
#import "Basic";
#import "Compiler";

Plugin :: struct {
    #as using base: Metaprogram_Plugin;
    text: String_Builder;
    received: bool;
    allocator: Allocator;
}

request_measurements :: (_p: *Metaprogram_Plugin, flags: *Intercept_Flags) {
    flags.* |= .DO_PERFORMANCE_REPORT_POLYMORPHS | .DO_PERFORMANCE_REPORT_RUNS;
}

collect_measurements :: (p: *Metaprogram_Plugin, message: *Message) {
    if !message || message.kind != .PERFORMANCE_REPORT return;

    state := cast(*Plugin) p;
    report := cast(*Message_Performance_Report) message;

    state.received = true;
    append(*state.text, "Compiler performance report\n");
    print_to_builder(*state.text, "#run time: % seconds\nBytecode generation: % seconds\nBytecode inlining: % seconds\n", report.time_report.run_directives, report.time_report.bytecode_generating, report.time_report.bytecode_inlining);
    print_to_builder(*state.text, "Bytecode instructions: % total, % inlined\nCalls: % total, % inlined\n", report.bytecode_report.num_instructions_total, report.bytecode_report.num_instructions_inlined, report.bytecode_report.num_calls_total, report.bytecode_report.num_calls_inlined);

    if !report.run_directive_report.reported {
        append(*state.text, "#run details unavailable: report flag disabled\n");
    } else {
        append(*state.text, "#run records (descending elapsed time)\n");
        rows: [..] Run_Directive_Report.Record;

        defer array_free(rows);
        for row: report.run_directive_report.records array_add(*rows, row);

        // Stable insertion sort keeps ties in producer order.
        for index: 1..rows.count - 1 {
            cursor := index;
            while cursor > 0 && rows[cursor - 1].elapsed_time < rows[cursor].elapsed_time {
                swap(*rows[cursor - 1], *rows[cursor]);
                cursor -= 1;
            }
        }

        for row: rows {
            name := ifx row.callee then row.callee.name else "<unknown>";
            print_to_builder(*state.text, "  %: % seconds, % stalls\n", name, row.elapsed_time, row.num_stalls);
        }
    }

    poly := report.polymorph_report;
    if !poly.reported append(*state.text, "Polymorph details unavailable: report flag disabled\n");
    else {
        print_to_builder(*state.text, "Polymorph solves: % invoked, % used\n", poly.num_solves_invoked, poly.num_solves_used);

        for row: poly.records {
            name := ifx row.source then row.source.name else "<unknown>";
            print_to_builder(*state.text, "  %: % call sites, % distinct, % constant deduplications, % bytecode deduplications\n", name, row.num_call_sites, row.distinct_polymorphs.count, row.num_deduplications_by_constants, row.num_deduplications_by_bytecode);
        }
    }
}

write_measurements :: (p: *Metaprogram_Plugin) {
    state := cast(*Plugin) p;
    if !state.received {
        compiler_report("Performance_Report: compiler did not provide a performance report", mode = .WARNING);
        return;
    }

    if state.text.failed {
        compiler_report("Performance_Report: output allocation failed");
        return;
    }

    buffer := get_base_buffer(*state.text);
    while buffer {
        print("%", string.{ buffer.count, get_buffer_data(buffer) });
        buffer = buffer.next;
    }
}

release_report :: (p: *Metaprogram_Plugin) {
    state := cast(*Plugin) p;
    free_buffers(*state.text);
    push_allocator(state.allocator);
    free(state);
}
