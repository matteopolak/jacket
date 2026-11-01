#import "Basic";
#import "Extensions/Testing";
Trace :: #import "Extensions/Iprof_Trace";
#import,dir "../model";

fixture :: () -> Trace.Trace {
    trace: Trace.Trace;

    array_add(*trace.names, copy_string("root"));
    array_add(*trace.names, copy_string("child"));
    frame: Trace.Frame;
    frame.duration = 100;
    array_add(*frame.spans, .{ zone = 0, start = 10, end = 90 });
    array_add(*frame.spans, .{ zone = 1, parent = 0, start = 20, end = 50 });
    array_add(*frame.spans, .{ zone = 0, parent = 0, start = 60, end = 80 });
    array_add(*trace.frames, frame);
    array_add(*trace.frames, Trace.Frame.{});

    return trace;
}

flame_uses_real_offsets_and_depth :: () {
    trace := fixture();

    defer Trace.free_trace(*trace);
    boxes := flame_layout(*trace.frames[0]);

    defer array_free(boxes);
    expect_equal(boxes.count, 3);
    expect_equal(boxes[0].depth, 0);
    expect_equal(boxes[2].depth, 1);
    expect(boxes[1].x > 0.199 && boxes[1].x < 0.201);
    expect(boxes[1].width > 0.299 && boxes[1].width < 0.301);
    expect_equal(frame_height(*trace, 0), 1.0);
    expect_equal(frame_height(*trace, 1), 0.0);
    empty := flame_layout(*trace.frames[1]);

    defer array_free(empty);
    expect_equal(empty.count, 0);
} @test

stats_separate_self_total_recursion_average_and_max :: () {
    trace := fixture();

    defer Trace.free_trace(*trace);
    stats := statistics(*trace, 0, "", .TOTAL);

    defer array_free(stats);
    require(stats.count == 2);
    expect_equal(stats[0].zone, 0);
    expect_equal(stats[0].calls, 2);
    expect_equal(stats[0].total, cast(u64) 100);
    expect_equal(stats[0].self_ns, cast(u64) 50);
    expect_equal(stats[0].average, 50.0);
    expect_equal(stats[0].maximum, cast(u64) 80);
    expect_equal(stats[1].self_ns, cast(u64) 30);
    expect_equal(stats[1].total, cast(u64) 30);
} @test

search_sort_and_empty_comparison :: () {
    trace := fixture();

    defer Trace.free_trace(*trace);

    for column: Sort_Column.[.NAME, .CALLS, .SELF, .TOTAL, .AVERAGE, .MAXIMUM] {
        ascending := statistics(*trace, 0, "", column, false);
        descending := statistics(*trace, 0, "", column, true);

        defer array_free(ascending);
        defer array_free(descending);
        expect_equal(ascending[0].zone, 1);
        expect_equal(descending[0].zone, 0);
    }

    filtered := statistics(*trace, 0, "ild");

    defer array_free(filtered);
    require(filtered.count == 1);
    expect_equal(filtered[0].zone, 1);
    missing := statistics(*trace, 0, "absent");

    defer array_free(missing);
    expect_equal(missing.count, 0);
    empty := statistics(*trace, 1, "", .NAME, false, include_empty = true);

    defer array_free(empty);
    expect_equal(empty.count, 2);
    for empty expect_equal(it.calls, 0);
} @test
