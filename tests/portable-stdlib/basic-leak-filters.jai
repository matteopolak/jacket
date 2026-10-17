B :: #import "Basic";
D :: #import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

// Storage obtained below the memory debugger so the test can record it with a synthetic trace.
unrecorded_alloc :: (size: s64) -> *void {
    allocator := context.default_allocator;
    return allocator.proc(.ALLOCATE, size, 0, null, allocator.data);
}

main :: () {
    infos: [7] Stack_Trace_Procedure_Info;
    infos[0] = .{ name = "allocate", location = .{ fully_pathed_filename = "/project/runtime/alloc.jai" } };
    infos[1] = .{ name = "site", location = .{ fully_pathed_filename = "/project/other/site.jai" } };
    infos[2] = .{ name = "site", location = .{ fully_pathed_filename = "/project/app/site.jai" } };
    infos[3] = .{ name = "site", location = .{ fully_pathed_filename = "/project/app/deeper.jai" } };
    infos[4] = .{ name = "skip", location = .{ fully_pathed_filename = "/project/app/middle.jai" } };
    infos[5] = .{ name = "keep", location = .{ fully_pathed_filename = "/project/app/main.jai" } };
    infos[6] = .{ name = "skip", location = .{ fully_pathed_filename = "/project/runtime/entry.jai" } };
    nodes: [8] Stack_Trace_Node;
    for *node, index: nodes {
        if index < infos.count node.info = *infos[index];
        node.line_number = cast(u32) (10 + index);
        if index + 1 < nodes.count node.next = *nodes[index + 1];
    }

    memory := unrecorded_alloc(24);

    memset(memory, 0, 24);
    D.add_allocation_to_table(memory, 24, context.allocator, nodes.data);
    leaves: [1] D.Procedure_To_Elide = .[.{ "site", "/app/" }];
    roots: [1] D.Procedure_To_Elide = .[.{ "skip", "" }];
    group_path := B.copy_string("/app/");
    D.this_allocation_is_not_a_leak(group_path.data);
    groups: [3] D.Procedure_To_Elide = .[.{ "sit", "/app/" }, .{ "site", "/APP/" }, .{ "site", group_path }];
    options := D.Leak_Report_Options.{ known_allocation_leaves = leaves, roots_to_skip = roots, group_by_these_calls = groups, scan_global_memory_to_remove_leaks = false, scan_leaks_to_combine = false };
    report := D.make_leak_report(options);

    B.assert(report.sorted_summaries.count == 1);
    summary := report.sorted_summaries[0];
    trace := summary.alloc_site_trace;

    B.assert(trace.nodes.count == 4);
    B.assert(trace.nodes[0].info == *infos[2] && trace.nodes[0].line_number == 12);
    B.assert(trace.nodes[2].info == *infos[4] && trace.nodes[3].info == *infos[5]);
    B.assert(trace.nodes[3].next == null);
    B.assert(summary.group_info != null && summary.group_info.procedure.module_name == "/app/");
    memset(group_path.data, 88, group_path.count);
    B.free(group_path);
    D.free(memory);
    nodes[2].next = null;
    B.assert(summary.group_info.procedure.module_name == "/app/");
    B.assert(trace.nodes[0].next == *trace.nodes[1] && trace.nodes[2].next == *trace.nodes[3]);
    B.assert(summary.bytes == 24 && summary.group_info.sub_summaries[0] == summary);
    D.deinit(*report);

    // A root filter cannot trim a matching interior frame beyond a nonmatch.
    memory = unrecorded_alloc(8);
    memset(memory, 0, 8);
    nodes[2].next = *nodes[3];
    D.add_allocation_to_table(memory, 8, context.allocator, nodes.data);
    leaves[0] = .{ "site", "/APP/" };
    options.group_by_these_calls = .{};
    report = D.make_leak_report(options);
    trace = report.sorted_summaries[0].alloc_site_trace;
    B.assert(trace.nodes.count == 6 && trace.nodes[0].info == *infos[0]);
    D.deinit(*report);
    // Repeated matches retain the first matching leaf, not the last.
    leaves[0] = .{ "site", "" };
    report = D.make_leak_report(options);
    trace = report.sorted_summaries[0].alloc_site_trace;
    B.assert(trace.nodes.count == 5 && trace.nodes[0].info == *infos[1]);
    D.deinit(*report);
    roots[0] = .{ "keep", "/app/" };
    report = D.make_leak_report(options);
    trace = report.sorted_summaries[0].alloc_site_trace;
    B.assert(trace.nodes.count == 6 && trace.nodes[5].info == *infos[6]);
    D.deinit(*report);
    // Root trimming consumes a matching suffix, including its null endpoint.
    suffix: [2] D.Procedure_To_Elide = .[.{ "skip", "" }, .{ "keep", "/app/" }];

    options.roots_to_skip = suffix;
    report = D.make_leak_report(options);
    trace = report.sorted_summaries[0].alloc_site_trace;
    B.assert(trace.nodes.count == 3 && trace.nodes[2].info == *infos[3]);
    D.deinit(*report);
    all_frames: [4] D.Procedure_To_Elide = .[.{ "allocate", "" }, .{ "site", "" }, .{ "skip", "" }, .{ "keep", "" }];

    options.roots_to_skip = all_frames;
    report = D.make_leak_report(options);
    B.assert(report.sorted_summaries[0].alloc_site_trace.nodes.count == 0);
    D.deinit(*report);
    // Missing locations cannot satisfy a path filter, even with matching names.
    infos[2].location.fully_pathed_filename = "";
    infos[3].location.fully_pathed_filename = "";
    options.roots_to_skip = roots;
    leaves[0] = .{ "site", "/app/" };
    missing_location: [1] D.Procedure_To_Elide = .[.{ "site", "/app/" }];

    options.group_by_these_calls = missing_location;
    report = D.make_leak_report(options);
    B.assert(report.sorted_summaries[0].group_info == null);
    B.assert(report.sorted_summaries[0].alloc_site_trace.nodes[0].info == *infos[0]);
    D.deinit(*report);
    D.free(memory);
    B.print("ok\n");
}
