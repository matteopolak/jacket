#import "Basic";
P :: #import "POSIX";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

pause :: () {
    duration: P.timespec;
    duration.tv_nsec = 3_000_000;
    assert(P.nanosleep(*duration, null) == 0);
}

main :: () {
    I.Prof_init_highlevel();
    assert(I.times_to_reach_90_percent[0] == 0.1 && I.times_to_reach_90_percent[1] == 0.8 && I.times_to_reach_90_percent[2] == 2.5);
    I.zones[0].name = "recursive measured";
    I.zones[0].initted = true;
    I.zones[1].name = "child measured";
    I.zones[1].initted = true;
    I.zone_begin(0);
    outer := I.stack;

    pause();
    I.zone_begin(0);
    recursive := I.stack;

    pause();
    I.zone_end();
    I.zone_begin(1);
    child := I.stack;

    pause();
    I.zone_end();
    I.zone_end();
    outer_ticks := outer.total_hier_ticks;
    self_ticks := outer.total_self_ticks + recursive.total_self_ticks;

    I.update(true);
    assert(I.update_index == 1 && I.history_index == 1);
    assert(outer.total_self_ticks == 0 && outer.total_hier_ticks == 0 && outer.total_entry_count == 0);
    assert(outer.profile_tracker_data_record != null && child.profile_tracker_data_record != null);
    report := I.create_report();

    assert(report != null && report.records.count == 2);
    found := false;

    for row: report.records if row.zone == *I.zones[0] {
        assert(row.calls == 2 && row.self_time > 0 && row.hier_time > 0);
        assert(row.self_time > 5 && row.hier_time > row.self_time);
        assert(row.self_time - cast(float64) self_ticks * I.timestamps_to_seconds * 1000 < 0.000001);
        assert(row.hier_time - cast(float64) outer_ticks * I.timestamps_to_seconds * 1000 < 0.000001);
        found = true;
    }

    assert(found && I.zones[0].highlevel != null);
    assert(I.zones[0].history_calls.values[0] == 2);
    I.zone_under_cursor = *I.zones[0];
    I.set_report_mode(.CALL_GRAPH);
    graph := I.create_report();

    assert(graph.records.count == 4);
    nested := false;
    for row: graph.records if row.indent == 1 && row.zone == *I.zones[0] nested = true;

    assert(nested);
    I.set_report_mode(.SELF_TIME);

    for frame: 1..130 {
        I.zone_begin(1);
        I.zone_end();
        I.update(true);
    }

    assert(I.update_index == 131 && I.history_index == 3);
    assert(I.frame_time.history[2] > 0);
    previous := I.get_value(*child.profile_tracker_data_record.entry_count);

    I.zone_begin(1);
    I.zone_end();
    I.update(false);
    assert(I.update_index == 131 && I.history_index == 3);
    assert(I.get_value(*child.profile_tracker_data_record.entry_count) == previous);
    assert(child.total_entry_count == 0);
    I.set_frame(100000);
    assert(I.display_frame == 127);
    I.set_frame(-1);
    assert(I.display_frame == 0);
    I.set_cursor(100000);
    assert(I.cursor >= 0);
    I.set_smoothing(0);
    report = I.create_report();
    assert(report.title.count > 0 && report.title_right == "current frame");
    assert(report.records.count > 0);
    print("ok\n");
}
