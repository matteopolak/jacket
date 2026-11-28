// Makes every allocation made so far exempt from leak reports, so a report lists only what leaked
// after this call. Does nothing unless Basic's `MEMORY_DEBUGGER` parameter is on.
//
// ```jai,only(macos, linux, windows)
// #import "Basic";
// D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
//
// main :: () {
//     setup := D.alloc(16);
//     D.allocations_before_this_call_will_not_be_considered_leaks();
//     report := D.make_leak_report();
//     defer D.deinit(*report);
//     assert(report.sorted_summaries.count == 0);
// }
// ```
allocations_before_this_call_will_not_be_considered_leaks :: () {
    #if MEMORY_DEBUGGER {
        debug_acquire();
        debug_baseline = debug_ordinal;
        debug_unlock();
    }
}

#if MEMORY_DEBUGGER {
    // Leaves the live allocation at `memory` out of leak reports, along with what it points to. Use it
    // for something kept until the program exits. Does nothing for null, an address that is not the
    // start of a tracked allocation, or when Basic's `MEMORY_DEBUGGER` parameter is off.
    //
    // ```jai,only(macos, linux, windows)
    // #import "Basic";
    // D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
    //
    // main :: () {
    //     D.allocations_before_this_call_will_not_be_considered_leaks();
    //     kept := D.alloc(16);
    //     D.this_allocation_is_not_a_leak(kept);
    //     report := D.make_leak_report();
    //     defer D.deinit(*report);
    //     assert(report.sorted_summaries.count == 0 && report.marked_as_non_leak_allocations > 0);
    // }
    // ```
    this_allocation_is_not_a_leak :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, false);
    }

    // Leaves `memory` out of leak reports as [this_allocation_is_not_a_leak] does, for a leak that
    // is known and not fixed yet, so a search finds the place later.
    this_allocation_leaks_todo :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, false);
    }

    // Keeps the allocation at `memory` in leak reports but stops the report from following its
    // pointers, so what it points to is judged on its own instead of as its child.
    this_allocation_does_not_propagate_leak_ownership :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, true);
    }
} else {
    this_allocation_is_not_a_leak :: (memory: *void) #expand #no_debug {}
    this_allocation_leaks_todo :: (memory: *void) #expand #no_debug {}
    this_allocation_does_not_propagate_leak_ownership :: (memory: *void) #expand #no_debug {}
}

// Records a new allocation with the memory debugger. Allocators that bypass `allocator_call` use
// it; most code never calls it.
add_allocation_to_table :: (memory: *void, size: s64, allocator: Allocator, trace: *Stack_Trace_Node) {
    assert(memory != null && size >= 0);
    debug_record(allocator, memory, size, false, trace);
}

// Tells the memory debugger that the tracked `allocation` has been freed.
do_free_bookkeeping :: (allocation: *Allocation) {
    assert(allocation != null);
    debug_release(allocation.owner, allocation.memory, allocation.heap, allocation.block);
}

// Counts `size` allocated bytes for `a` in the memory visualizer's current frame.
add_allocation_this_frame :: (a: *Allocation, size: s64) {
    assert(a != null && size >= 0);
    debug_acquire();
    a.frame_allocations += 1;
    a.frame_bytes += size;
    frame_allocations += 1;
    frame_allocated_bytes += cast(u64) size;
    #if MEMORY_DEBUGGER visualization_note_locked(a, false, size);
    debug_unlock();
}

// Counts `size` freed bytes for `a` in the memory visualizer's current frame.
add_free_this_frame :: (a: *Allocation, size: s64) {
    assert(a != null && size >= 0);
    debug_acquire();
    frame_frees += 1;
    frame_freed_bytes += cast(u64) size;
    #if MEMORY_DEBUGGER visualization_note_locked(a, true, size);
    debug_unlock();
}

// Finds the live allocations nothing reachable points to and groups them by where they were made.
//
// Allocations made before [allocations_before_this_call_will_not_be_considered_leaks] or marked
// with [this_allocation_is_not_a_leak] are counted apart, not listed. The report allocates with the
// context allocator; free it with [deinit]. It takes about `O(n log n)` time in the number of
// live allocations, plus a scan of global data and of every leaked byte. [report_memory_leaks]
// makes one and logs it.
//
// - for_visualizer: skips the scans and the exemptions and counts this frame's allocations, for the
//   memory viewer.
//
// ```jai,only(macos, linux, windows)
// #import "Basic";
// D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
//
// main :: () {
//     D.allocations_before_this_call_will_not_be_considered_leaks();
//     lost := D.alloc(24);
//     report := D.make_leak_report();
//     defer D.deinit(*report);
//     assert(report.sorted_summaries.count == 1 && report.sorted_summaries[0].bytes == 24);
//     D.free(lost);
// }
// ```
make_leak_report :: (options: Leak_Report_Options = .{}, for_visualizer := false) -> Leak_Report {
    #if MEMORY_DEBUGGER {
        debug_acquire();
        defer debug_unlock();

        return build_leak_report_locked(options, for_visualizer);
    } else {
        return .{};
    }
}

#scope_file
RP :: #import "Pool";
RT :: #import "Hash_Table";

#scope_export
// Frees what [make_leak_report] allocated for `report` and clears it.
deinit :: (report: *Leak_Report) {
    table_pool := ifx report.leak_combiner_table.allocator.proc == RP.pool_allocator_proc then cast(*RP.Pool) report.leak_combiner_table.allocator.data else null;

    RT.deinit(*report.leak_combiner_table);
    if table_pool RP.release(table_pool);

    RP.release(*report.pool);
    report.* = .{};
}

// Logs each summary of `report` with its trace, then the totals, through `context.logger`. Pass the
// `options` the report was made with so traces are trimmed the same way.
log_leak_report :: (report: Leak_Report, options: Leak_Report_Options = .{}) {
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count + summary.child_count;
        bytes += summary.bytes + summary.child_bytes;
    }

    #if MEMORY_DEBUGGER {
        for summary: report.sorted_summaries {
            if summary.group_info log("Group: %\n", summary.group_info.procedure.procedure_name);
            else for procedure: options.group_by_these_calls {
                one := ([] Procedure_To_Elide).{ 1, *procedure };
                if trace_matches(summary.alloc_site_trace, one) {
                    log("Group: %\n", procedure.procedure_name);
                    break;
                }
            }

            log_error("  % allocations / % bytes; % children / % bytes\n", summary.count, summary.bytes, summary.child_count, summary.child_bytes);

            if summary.alloc_site_trace {
                begin, end := trace_bounds(summary.alloc_site_trace, options);
                for index: begin..end - 1 {
                    node := summary.alloc_site_trace.nodes[index];
                    if node.info log_error("    '%' at %:%\n", node.info.name, node.info.location.fully_pathed_filename, node.line_number);
                }
            }
        }
    }

    log_error("Memory leaks: % allocations / % bytes\n", count, bytes);
    log("Marked non-leaks: % allocations / % bytes\n", report.marked_as_non_leak_allocations, report.marked_as_non_leak_bytes);
}

// Makes a leak report, logs it and frees it: [make_leak_report], then [log_leak_report]. Call it
// at the end of `main`. Without Basic's `MEMORY_DEBUGGER` parameter it logs zero leaks.
//
// ```jai,only(macos, linux, windows)
// #import "Basic";
// D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
//
// main :: () {
//     D.allocations_before_this_call_will_not_be_considered_leaks();
//     D.report_memory_leaks(); // "Memory leaks: 0 allocations / 0 bytes"
// }
// ```
report_memory_leaks :: (options: Leak_Report_Options = .{}) {
    report := make_leak_report(options);
    defer deinit(*report);
    log_leak_report(report, options);
}
