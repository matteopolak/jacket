log_text_report :: (report: *Report, min_self_time := 0., min_hier_time := 0.) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(report != null);
    builder: String_Builder;

    defer free_buffers(*builder);
    print_to_builder(*builder, "% %\n", report.title, report.title_right);
    fallback: [5] string = .["zone", "self (ms)", "hier (ms)", "calls", "heat"];

    for title, index: report.header {
        if index > 0 append(*builder, " | ");
        append(*builder, ifx title then title else fallback[index]);
    }

    append(*builder, "\n");

    for row: report.records {
        if !report_row_visible(row) || (min_self_time > 0 && row.self_time < min_self_time) || (min_hier_time > 0 && row.hier_time < min_hier_time) continue;
        for 0..max(cast(s64) 0, row.indent) - 1 append(*builder, "  ");
        if row.prefix append(*builder, string.{ 1, *row.prefix });
        print_to_builder(*builder, "% | % | % | % | %\n", row.name, row.self_time, row.hier_time, row.calls, row.heat);
    }

    if builder.failed {
        log_error("Iprof: text report allocation failed\n");
        return;
    }

    text := builder_to_string(*builder);

    defer free(text);
    log("%", text);
}

save_csv_report :: (report: *Report, filename: string) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(report != null);
    builder: String_Builder;

    defer free_buffers(*builder);
    arguments := get_command_line_arguments();

    defer free(arguments.data);
    append(*builder, "Command:");
    for argument: arguments print_to_builder(*builder, " %", argument);

    append(*builder, "\nname,self (ms),hier (ms),calls\n");

    for row: report.records {
        if !report_row_visible(row) continue;
        append_csv_field(*builder, row.name);
        print_to_builder(*builder, ",%,%,%\n", row.self_time, row.hier_time, row.calls);
    }

    if builder.failed {
        log_error("Iprof: CSV report allocation failed\n");
        return;
    }

    if !write_entire_file(filename, *builder, false) log_error("Iprof: could not write CSV report %\n", filename);
}

#scope_file
#import "Basic";
#import "File";

report_row_visible :: (row: Report_Row) -> bool {
    if row.calls == 0 return false;

    #if MANUAL_MODE {
        if !row.zone || !row.zone.initted return false;
    }

    return true;
}

append_csv_field :: (builder: *String_Builder, field: string) {
    append(builder, "\"");

    for c: field {
        if c == #char "\"" append(builder, "\"");
        append(builder, string.{ 1, *c });
    }

    append(builder, "\"");
}
