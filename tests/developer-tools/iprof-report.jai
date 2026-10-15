#import "Basic";
#import "File";
I :: #import "Iprof"(USE_GRAPHICS = false);
M :: #import "Iprof"(MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);

#assert size_of(I.Zone) - size_of(M.Zone) == 1536;

captured: string;

capture :: (text: string, _data: *void, _info: Log_Info) {
    free(captured);
    captured = copy_string(text);
}

main :: () {
    row: I.Report_Row;
    row.self_time = 0.25;
    row.array_aliased_values[1] = 0.75;
    assert(row.array_aliased_values[0] == row.self_time && row.hier_time == 0.75);
    report: I.Report;
    report.title = "Original report";
    array_add(*report.records, I.Report_Row.{ name = "quoted \"zone\",\nname", self_time = 0.25, hier_time = 0.5, calls = 3, heat = 0.75 });
    array_add(*report.records, I.Report_Row.{ name = "zero calls", self_time = 9, hier_time = 9 });
    defer array_free(report.records);
    path := tprint("%../../target/developer-tools-iprof.csv", #filepath);

    I.save_csv_report(*report, path);
    csv, ok := read_entire_file(path);

    defer free(csv);
    assert(ok && csv == "name,self_time,hier_time,calls,heat\n\"quoted \"\"zone\"\",\nname\",0.25,0.5,3,0.75\n");

    push_context {
        context.logger = capture;
        I.log_text_report(*report);
    }

    defer free(captured);
    assert(captured.count > 0);
    assert(contains(captured, "quoted") && !contains(captured, "zero calls"));

    push_context {
        context.logger = capture;
        I.log_text_report(*report, min_self_time = 0.3);
    }

    assert(!contains(captured, "quoted"));
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
    assert(filtered_ok && filtered == "name,self_time,hier_time,calls,heat\n\"initialized\",0,0,4,0\n");
    print("ok\n");
}

#scope_file
#import "String";
