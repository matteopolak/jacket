#import "Basic";
#import "File";
I :: #import "Iprof"(USE_GRAPHICS = false);
M :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);

#assert size_of(I.Zone) - size_of(M.Zone) == 1536;

captured: string;
errors: s64;
csv_failures: s64;

capture :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
    if info.common_flags & .ERROR && contains(text, "Iprof: could not write CSV report") csv_failures += 1;
    free(captured);
    captured = copy_string(text);
}

main :: () {
    row: I.Report_Row;
    row.self_time = 250;
    row.array_aliased_values[1] = 0.75;
    assert(row.array_aliased_values[0] == row.self_time && row.hier_time == 0.75);
    report: I.Report;
    report.title = "Original report";
    array_add(*report.records, I.Report_Row.{ name = "quoted \"zone\",\nname", self_time = 250, hier_time = 500, calls = 3, heat = 0.75 });
    array_add(*report.records, I.Report_Row.{ name = "zero calls", self_time = 9, hier_time = 9 });
    defer array_free(report.records);
    path := tprint("%../../../../target/developer-tools-iprof.csv", #filepath);

    I.save_csv_report(*report, path);
    csv, ok := read_entire_file(path);

    defer free(csv);
    assert(ok && contains(csv, "\nname,self (ms),hier (ms),calls\n\"quoted \"\"zone\"\",\nname\",250,500,3\n"));
    assert(starts_with(csv, "Command:"));

    push_context {
        context.logger = capture;
        I.log_text_report(*report);
    }

    defer free(captured);
    assert(captured.count > 0);
    assert(contains(captured, "quoted") && !contains(captured, "zero calls"));
    assert(contains(captured, "250 | 500 | 3"));

    push_context {
        context.logger = capture;
        I.log_text_report(*report, min_self_time = 300);
    }

    assert(!contains(captured, "quoted"));

    push_context {
        context.logger = capture;
        I.log_text_report(*report, min_hier_time = 600);
    }

    assert(!contains(captured, "quoted"));

    push_context {
        context.logger = capture;
        I.log_text_report(*report, min_self_time = 250, min_hier_time = 500);
    }

    assert(contains(captured, "quoted"));
    array_add(*report.records, I.Report_Row.{ name = "negative measurement", self_time = -1, hier_time = -2, calls = 1 });

    push_context {
        context.logger = capture;
        I.log_text_report(*report, min_self_time = -0.1, min_hier_time = -0.1);
    }

    assert(contains(captured, "negative measurement"));
    manual: M.Report;
    uninitialized: M.Zone;
    initialized: M.Zone;
    initialized.initted = true;
    array_add(*manual.records, M.Report_Row.{ name = "uninitialized", calls = 7, zone = *uninitialized });
    array_add(*manual.records, M.Report_Row.{ name = "initialized", calls = 4, zone = *initialized });
    defer array_free(manual.records);
    M.save_csv_report(*manual, path);
    filtered, filtered_ok := read_entire_file(path);

    defer free(filtered);
    assert(filtered_ok && contains(filtered, "\nname,self (ms),hier (ms),calls\n\"initialized\",0,0,4\n"));
    assert(!contains(filtered, "uninitialized"));

    push_context {
        context.logger = capture;
        I.save_csv_report(*report, tprint("%../../../../target/developer-tools-iprof-absent-parent/report.csv", #filepath));
    }

    assert(errors >= 1 && csv_failures == 1 && contains(captured, "could not write CSV report"));
    print("ok\n");
}

#scope_file
#import "String";
