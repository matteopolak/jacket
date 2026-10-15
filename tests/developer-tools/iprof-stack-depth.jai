#import "Basic";
I :: #import "Iprof"(USE_GRAPHICS = false);

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    zone, middle_zone, other_zone: I.Zone;
    zone.name = "same spelling";
    other_zone.name = zone.name;
    bottom, middle, next, top: I.Zone_Profiling_Data;
    bottom.zone = *zone;
    middle.zone = *middle_zone;
    middle.parent = *bottom;
    next.zone = *zone;
    next.parent = *middle;
    top.zone = *zone;
    top.parent = *next;
    assert(I.count_recursion_depth(null, *zone) == 0);
    assert(I.count_recursion_depth(*top, *zone) == 3);
    assert(I.count_recursion_depth(*top, *middle_zone) == 1);
    assert(I.count_recursion_depth(*top, *other_zone) == 0);
    assert(I.count_recursion_depth(*top, null) == 0);
    assert(I.count_recursion_depth(*bottom, *zone) == 1);
    bottom.parent = *top;

    push_context {
        context.logger = capture;
        assert(I.count_recursion_depth(*top, *zone) == -1);
    }

    assert(errors == 1);
    bottom.parent = null;
    assert(I.count_recursion_depth(*top, *zone) == 3);
    tracker: I.Profile_Tracker_Data_Record;
    tracker.self_time.values = .[1, 2, 3];
    tracker.hierarchical_time.values = .[4, 5, 6];
    tracker.entry_count.values = .[7, 8, 9];
    top.profile_tracker_data_record = *tracker;
    assert(I.get_value(*top.profile_tracker_data_record.self_time) == 2);
    assert(I.get_value(*top.profile_tracker_data_record.hierarchical_time) == 5);
    assert(I.get_value(*top.profile_tracker_data_record.entry_count) == 8);
    assert(top.parent == *next && top.zone == *zone && bottom.parent == null);
    print("ok\n");
}
