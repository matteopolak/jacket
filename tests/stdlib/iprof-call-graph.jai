// Iprof runtime (MANUAL_MODE): call counts per frame, recursion folding, the call-graph view,
// history frames, and the text/CSV writers.
#import "Basic";
#import "File";
#import "String";
Iprof :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

LEAF   :: 1;
MIDDLE :: 2;
TOP    :: 3;

leaf :: () {
    Iprof.zone_begin(LEAF);
    defer Iprof.zone_end();
}

middle :: (depth: int) {
    Iprof.zone_begin(MIDDLE);
    defer Iprof.zone_end();
    leaf();
    leaf();
    if depth > 0  middle(depth - 1);  // Direct recursion folds into one node.
}

top :: () {
    Iprof.zone_begin(TOP);
    defer Iprof.zone_end();
    middle(2);
    leaf();
}

find_row :: (report: *Iprof.Report, name: string) -> *Iprof.Report_Row {
    for * report.records  if it.name == name  return it;
    return null;
}

main :: () {
    Iprof.zones[0].name      = "(root)";
    Iprof.zones[LEAF].name   = "leaf";
    Iprof.zones[MIDDLE].name = "middle";
    Iprof.zones[TOP].name    = "top";
    Iprof.zones[TOP].full_path = "demo/top.jai";
    Iprof.zones[TOP].line      = 7;

    Iprof.init_runtime();
    Iprof.set_smoothing(0);  // Latest frame only, so counts are exact.

    for 1..4 {
        top();
        Iprof.update(true);
    }
    // A frame with nothing recorded must not disturb the averages.
    top();
    Iprof.update(false);

    report := Iprof.create_report();
    assert(report.records.count == 4);
    assert(find_row(report, "leaf").calls   == 7);  // 2 per middle level (3 levels) + 1 from top.
    assert(find_row(report, "middle").calls == 3);
    assert(find_row(report, "top").calls    == 1);
    for report.records  assert(it.self_time <= it.hier_time + 0.0001);

    // Sorted by self time, largest first.
    for 1..report.records.count-1  assert(report.records[it - 1].self_time >= report.records[it].self_time);

    Iprof.set_report_mode(.CALL_COUNT);
    report = Iprof.create_report();
    assert(report.records[0].name == "leaf" && report.hilit_header == 3);

    // Focus the call graph on `middle`: its caller above, itself, its callee below.
    Iprof.set_cursor(1);  // `middle` is second by call count.
    Iprof.select();
    report = Iprof.create_report();
    assert(Iprof.displayed_quantity == .CALL_GRAPH);
    assert(report.records.count == 3, "%", report.records.count);
    assert(report.records[0].prefix == #char "<" && report.records[0].name == "top");
    assert(report.records[1].prefix == #char "*" && report.records[1].name == "middle");
    assert(report.records[2].prefix == #char ">" && report.records[2].name == "leaf");
    assert(report.records[2].calls == 6);
    assert(report.hilight == 1);  // The cursor lands on the focused zone.

    // Back up to the caller, then out of the call graph from the root.
    Iprof.select_parent();
    report = Iprof.create_report();
    assert(report.records[1].name == "top");
    Iprof.select_parent();
    Iprof.select_parent();
    assert(Iprof.displayed_quantity == .SELF_TIME);

    // History: the oldest kept frame.
    Iprof.set_frame(100);
    assert(Iprof.display_frame == 3);
    report = Iprof.create_report();
    assert(find_row(report, "leaf").calls == 7);
    assert(contains(report.title_right, "-3"));
    Iprof.set_frame(0);

    // Text report: title, header, one line per zone.
    report = Iprof.create_report();
    text := Iprof.text_report(report);
    lines := split(text, "\n");
    assert(begins_with(lines[0], "Iprof: self time"), "%", text);
    assert(contains(lines[1], "self ms") && contains(lines[1], "hier ms"), "%", text);
    leaf_line := "";
    for lines  if ends_with(it, " leaf")  leaf_line = it;
    assert(contains(leaf_line, " 7 "), "%", text);

    // CSV report.
    path := "iprof-call-graph-test.csv";
    assert(Iprof.save_csv_report(report, path));
    csv, ok := read_entire_file(path);
    assert(ok);
    file_delete(path);
    assert(begins_with(csv, "zone,self_ms,hier_ms,calls,indent,file,line\n"), "%", csv);
    assert(contains(csv, "\"top\","), "%", csv);
    assert(contains(csv, ",\"demo/top.jai\",7\n"), "%", csv);

    print("ok\n");
}
