B :: #import "Basic";
D :: #import "Basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
S :: #import "String";

capture :: (message: string, data: *void, info: Log_Info) {
    B.append(cast(*B.String_Builder) data, message);
}

main :: () {
    parent := D.alloc(16);
    child := D.alloc(32);

    memset(parent, 0, 16);
    memset(child, 0, 32);
    (cast(**void) parent).* = cast(*u8) child + 1;
    options := D.Leak_Report_Options.{ scan_global_memory_to_remove_leaks = false };
    report := D.make_leak_report(options);

    B.assert(report.sorted_summaries.count == 1);
    B.assert(report.sorted_summaries[0].count == 1 && report.sorted_summaries[0].bytes == 16);
    B.assert(report.sorted_summaries[0].child_count == 1 && report.sorted_summaries[0].child_bytes == 32);
    D.deinit(*report);
    (cast(**void) child).* = parent;
    report = D.make_leak_report(options);
    B.assert(report.sorted_summaries.count == 1 && report.sorted_summaries[0].child_count == 1);
    D.deinit(*report);
    memset(child, 0, 32);
    D.this_allocation_does_not_propagate_leak_ownership(parent);
    report = D.make_leak_report(options);
    B.assert(report.sorted_summaries.count == 2);
    B.assert(report.sorted_summaries[0].bytes == 32 && report.sorted_summaries[1].bytes == 16);
    D.deinit(*report);
    D.free(parent);
    D.free(child);

    leaf_info := Stack_Trace_Procedure_Info.{ name = "allocator_leaf", location = .{ fully_pathed_filename = "alloc.jai" } };
    site_info := Stack_Trace_Procedure_Info.{ name = "user_site", location = .{ fully_pathed_filename = "app.jai" } };
    root_info := Stack_Trace_Procedure_Info.{ name = "root_caller", location = .{ fully_pathed_filename = "main.jai" } };
    root := Stack_Trace_Node.{ info = *root_info };
    site := Stack_Trace_Node.{ info = *site_info, next = *root };
    leaf := Stack_Trace_Node.{ info = *leaf_info, next = *site };
    memory := B.alloc(24);

    memset(memory, 0, 24);
    D.add_allocation_to_table(memory, 24, context.allocator, *leaf);
    leaves: [1] D.Procedure_To_Elide = .[.{ procedure_name = "allocator_leaf" }];
    roots: [1] D.Procedure_To_Elide = .[.{ procedure_name = "root_caller" }];
    groups: [1] D.Procedure_To_Elide = .[.{ procedure_name = "user_site" }];

    options.known_allocation_leaves = leaves;
    options.roots_to_skip = roots;
    options.group_by_these_calls = groups;
    report = D.make_leak_report(options);
    B.assert(report.sorted_summaries.count == 1 && report.sorted_summaries[0].bytes == 24);
    group := report.sorted_summaries[0].group_info;

    B.assert(group != null && group.procedure.procedure_name == "user_site");
    B.assert(group.sub_summaries.count == 1 && group.sub_summaries[0] == report.sorted_summaries[0]);
    builder: B.String_Builder;

    defer B.free_buffers(*builder);

    push_context {
        context.logger = capture;
        context.logger_data = *builder;
        D.log_leak_report(report, options);
    }

    text := B.builder_to_string(*builder);

    defer B.free(text);
    B.assert(S.contains(text, "user_site") && !S.contains(text, "allocator_leaf") && !S.contains(text, "root_caller"));
    B.assert(S.contains(text, "24 bytes") && S.contains(text, "Marked non-leaks"));
    D.free(memory);
    // Report storage remains valid after the tracked object and its metadata die.
    B.assert(report.sorted_summaries[0].bytes == 24 && group.sub_summaries.count == 1);
    D.deinit(*report);
    B.print("ok\n");
}
