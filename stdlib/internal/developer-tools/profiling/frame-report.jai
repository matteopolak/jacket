compute_heat :: (variance: float64, value: float64) -> float64 {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    if Profile_Math.abs(value) < 0.000000001 return 0;

    variance = max(cast(float64) 0, variance - value * value);
    ratio := Profile_Math.sqrt(variance) / Profile_Math.abs(value);

    return min(cast(float64) 1, ratio);
}

report_compare_slot :: (p: Report_Row, q: Report_Row, slot: s64) -> s64 {
    assert(slot >= 0 && slot < NUM_VALUES);
    a := ifx slot == 3 then p.heat else p.array_aliased_values[slot];
    b := ifx slot == 3 then q.heat else q.array_aliased_values[slot];

    if a > b return -1;
    if a < b return 1;
    if p.name.count < q.name.count return -1;
    if p.name.count > q.name.count return 1;

    for i: 0..p.name.count - 1 {
        if p.name[i] < q.name[i] return -1;
        if p.name[i] > q.name[i] return 1;
    }

    return 0;
}

report_expand_compare :: (a: Report_Row, b: Report_Row) -> s64 {
    a_group := profile_row_group(a);
    b_group := profile_row_group(b);

    if a_group < b_group return -1;
    if a_group > b_group return 1;
    if a.hier_time == b.hier_time return 0;
    if a_group == 0 return ifx a.hier_time < b.hier_time then -1 else 1;

    return ifx a.hier_time > b.hier_time then -1 else 1;
}

propagate_to_zone :: (c: *Zone_Profiling_Data) {
    assert(c != null && c.zone != null);
    if !c.profile_tracker_data_record || !c.zone.highlevel return;

    tracker := c.profile_tracker_data_record;
    row := c.zone.highlevel;
    row.self_time += 1000 * profile_history_value(*tracker.self_time);
    row.hier_time += 1000 * profile_history_value(*tracker.hierarchical_time);
    row.calls += profile_history_value(*tracker.entry_count);
    row.heat = max(row.heat, compute_heat(tracker.self_time.variances[min(max(slot, cast(s64) 0), cast(s64) NUM_SMOOTHING_SLOTS - 1)], get_value(*tracker.self_time)));
}

propagate_expanded :: (c: *Zone_Profiling_Data) {
    assert(c != null);
    profile_add_graph(c);
}

create_report :: () -> *Report {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    Prof_init_highlevel();
    push_allocator(temporary_allocator);
    profile_report = New(Report);
    for c: zone_hash if c && c.zone c.zone.highlevel = null;

    array_resize(*profile_report.records, zone_hash_count * 3);
    profile_report.records.count = 0;
    average := get_value(*frame_time);

    profile_report.title = tprint("% ms/frame (% FPS)", average * 1000, ifx average > 0 then 1 / average else 0.0);
    profile_report.title_right = ifx display_frame == 0 then "current frame" else tprint("% frames ago", display_frame);
    profile_report.header = .["", "self (ms)", "hier (ms)", "calls", "heat"];
    profile_report.hilit_header = cast(s32) (profile_report_slot() + 1);

    if displayed_quantity == .CALL_GRAPH {
        if !zone_under_cursor {
            for c: zone_hash if c && c.zone {
                zone_under_cursor = c.zone;
                break;
            }
        }

        for c: zone_hash if c && c.zone {
            existing := false;

            for row: profile_report.records if row.zone == c.zone {
                existing = true;
                break;
            }

            if existing continue;

            array_add(*profile_report.records, Report_Row.{ name = c.zone.name, zone = c.zone, prefix = #char "<" });
            array_add(*profile_report.records, Report_Row.{ name = c.zone.name, zone = c.zone, prefix = #char ">", indent = 1 });
            array_add(*profile_report.records, Report_Row.{ name = c.zone.name, zone = c.zone, prefix = #char "-" });
        }

        traverse(propagate_expanded);
        output: s64;

        for row: profile_report.records if profile_report_visible(row) {
            profile_report.records[output] = row;
            output += 1;
        }

        profile_report.records.count = output;

        for i: 1..output - 1 {
            j := i;
            while j > 0 && report_expand_compare(profile_report.records[j], profile_report.records[j - 1]) < 0 {
                swap := profile_report.records[j];
                profile_report.records[j] = profile_report.records[j - 1];
                profile_report.records[j - 1] = swap;
                j -= 1;
            }
        }
    } else {
        for c: zone_hash if c && c.zone && !c.zone.highlevel {
            array_add(*profile_report.records, Report_Row.{ name = c.zone.name, zone = c.zone });
            c.zone.highlevel = *profile_report.records[profile_report.records.count - 1];
        }

        traverse(propagate_to_zone);
        output: s64;

        for row: profile_report.records {
            if !profile_report_visible(row) continue;
            profile_report.records[output] = row;
            output += 1;
        }

        profile_report.records.count = output;

        for i: 1..output - 1 {
            j := i;
            while j > 0 && report_compare_slot(profile_report.records[j], profile_report.records[j - 1], profile_report_slot()) < 0 {
                temp := profile_report.records[j];
                profile_report.records[j] = profile_report.records[j - 1];
                profile_report.records[j - 1] = temp;
                j -= 1;
            }
        }
    }

    num_active_zones = profile_report.records.count;
    for c: zone_hash if c && c.zone c.zone.highlevel = null;

    for *row, i: profile_report.records {
        row.number = i + 1;
        if !row.zone.highlevel row.zone.highlevel = row;
    }

    cursor = max(cast(s64) 0, min(cursor, profile_report.records.count - 1));
    next_zone_under_cursor = ifx profile_report.records.count > 0 then profile_report.records[cursor].zone else null;
    profile_report.hilight = cursor;

    return profile_report;
}

set_cursor :: (num: s64) {
    cursor = num;
}

move_cursor :: (num: s64) {
    if num > 0 && cursor > 0x7fffffffffffffff - num cursor = 0x7fffffffffffffff;
    else if num < 0 && cursor < (-0x7fffffffffffffff - 1) - num cursor = -0x7fffffffffffffff - 1;
    else cursor += num;
}

set_frame :: (num: s64) {
    display_frame = max(cast(s64) 0, min(num, cast(s64) NUM_FRAME_SLOTS - 1));
}

move_frame :: (delta: s64) {
    if delta < -NUM_FRAME_SLOTS set_frame(NUM_FRAME_SLOTS - 1);
    else if delta > NUM_FRAME_SLOTS set_frame(0);
    else set_frame(display_frame - delta);
}

select :: () {
    if !profile_report || profile_report.records.count == 0 return;
    index := max(cast(s64) 0, min(cursor, profile_report.records.count - 1));
    zone_under_cursor = profile_report.records[index].zone;
    displayed_quantity = .CALL_GRAPH;
}

select_parent :: () {
    if !profile_report return;

    for row: profile_report.records {
        if row.prefix == #char "-" && row.zone == zone_under_cursor break;
        if row.zone != zone_under_cursor next_zone_under_cursor = row.zone;
    }

    if next_zone_under_cursor zone_under_cursor = next_zone_under_cursor;

    update_cursor = true;
}

#scope_module
#import "Basic";

profile_report: *Report;

profile_report_slot :: () -> s64 {
    if displayed_quantity == .HIERARCHICAL_TIME return 1;
    if displayed_quantity == .CALL_COUNT return 2;
    return 0;
}

profile_report_visible :: (row: Report_Row) -> bool {
    if row.calls == 0 return false;
    #if MANUAL_MODE if !row.zone || !row.zone.initted return false;
    return true;
}

profile_row_group :: (row: Report_Row) -> s64 {
    if row.prefix == #char "<" return 0;
    if row.prefix == #char ">" return 1;
    return 2;
}

profile_graph_row :: (zone: *Zone, prefix: u8) -> *Report_Row {
    for *row: profile_report.records if row.zone == zone && row.prefix == prefix return row;
    return null;
}

profile_add_graph :: (c: *Zone_Profiling_Data) {
    if !c.profile_tracker_data_record || !c.zone return;

    tracker := c.profile_tracker_data_record;
    calls := profile_history_value(*tracker.entry_count);
    self_time := 1000 * profile_history_value(*tracker.self_time);
    hierarchical := 1000 * profile_history_value(*tracker.hierarchical_time);

    if c.zone == zone_under_cursor {
        row := profile_graph_row(c.zone, #char "-");
        if row {
            row.self_time += self_time;
            row.hier_time += hierarchical;
            row.calls += calls;
        }

        if c.parent && c.parent.zone {
            parent := profile_graph_row(c.parent.zone, #char "<");
            if parent {
                parent.hier_time += hierarchical;
                parent.calls += calls;
            }
        }
    }

    if c.parent && c.parent.zone == zone_under_cursor {
        child := profile_graph_row(c.zone, #char ">");
        if child {
            child.self_time += self_time;
            child.hier_time += hierarchical;
            child.calls += calls;
        }
    }
}
