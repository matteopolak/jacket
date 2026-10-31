// Frame timing with Iprof's manual mode: each phase of a frame is a numbered zone, and every
// frame ends with `profile_frame_end`, which folds the zones into the history. `profile_report`
// prints the self-time table. Only the main thread opens zones (the zone stack is global).

Zone :: enum s64 {
    FRAME;
    INPUT;
    UPDATE;
    CULL;
    PACK;
    DRAW;
    PRESENT;
}

zone_names :: string.["frame", "input", "update", "cull", "pack", "draw", "present"];

profile_init :: () {
    I.Prof_init_highlevel();

    for name: zone_names {
        I.zones[it_index].name = name;
        I.zones[it_index].hash = cast(u64) (it_index * 7919 + 13);
        I.zones[it_index].initted = true;
    }
}

zone_begin :: (zone: Zone) {
    I.zone_begin(cast(s64) zone);
}

zone_end :: () {
    I.zone_end();
}

profile_frame_end :: () {
    I.update(true);
}

// Prints the zones of the last frames, slowest first, to the log.
profile_report :: () {
    report := I.create_report();

    I.log_text_report(report);
}

// How many times a zone was entered during the last frame (0 when it never ran).
profile_zone_calls :: (zone: Zone) -> float64 {
    report := I.create_report();

    for row: report.records {
        if row.zone == *I.zones[cast(s64) zone] return row.calls;
    }

    return 0;
}
