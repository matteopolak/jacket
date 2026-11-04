#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import "Extensions/Iprof_Trace";
I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);

captures_recursive_calls_across_frames :: (t: *Test) {
    r: Recorder;

    start_capture(*r);
    I.zones[0].name = "recursive";
    I.zones[0].initted = true;
    I.zone_begin(0);
    I.zone_begin(0);
    I.update(true);
    I.zone_end();
    I.zone_end();
    I.update(true);
    stop_capture(*r);
    defer free_trace(*r.trace);
    expect_equal(r.trace.frames.count, 2);

    for r.trace.frames {
        expect_equal(it.spans.count, 2);
        expect_equal(it.spans[1].parent, 0);
        expect(it.spans[1].end <= it.spans[0].end);
    }

    path := tprint("%/trace.jipt", temp_dir(t));

    expect(save_trace(*r.trace, path));
    loaded, ok := load_trace(path);

    defer free_trace(*loaded);
    expect(ok);
    expect_equal(loaded.names[0], "recursive");
    expect_equal(loaded.frames.count, 2);
    expect_equal(loaded.frames[1].duration, r.trace.frames[1].duration);
    expect_equal(loaded.frames[0].spans[1].end, r.trace.frames[0].spans[1].end);
} @test @skip_on(.WASM)

rejects_truncation_trailing_bytes_and_bad_trees :: (t: *Test) {
    trace: Trace;

    array_add(*trace.names, copy_string("zone"));
    frame: Frame;
    frame.duration = 100;
    array_add(*frame.spans, .{ zone = 0, start = 10, end = 90 });
    array_add(*trace.frames, frame);
    defer free_trace(*trace);
    path := tprint("%/valid.jipt", temp_dir(t));

    require(save_trace(*trace, path));
    bytes, ok := read_entire_file(path);

    require(ok);
    defer free(bytes);

    for length: 0..bytes.count - 1 {
        decoded, valid := decode_trace(slice(bytes, 0, length));
        free_trace(*decoded);
        expect(!valid);
    }

    decoded, valid := decode_trace(tprint("%x", bytes));

    free_trace(*decoded);
    expect(!valid);

    // This fixture's span begins at byte 52; mutate each independent wire invariant.
    for offset, index: s64.[0, 8, 16, 52, 60, 68, 76] {
        changed := copy_string(bytes);
        changed[offset] = u8.[0, 255, 255, 1, 1, 95, 101][index];
        invalid, accepted := decode_trace(changed);

        free_trace(*invalid);
        free(changed);
        expect(!accepted);
    }

    trace.frames[0].spans[0].end = 101;
    expect(!save_trace(*trace, path));
    trace.frames[0].spans[0].end = 90;
    trace.frames[0].spans[0].parent = 0;
    expect(!save_trace(*trace, path));
} @test

reports_work_with_and_without_capture :: () {
    I.zones[1].name = "report zone";
    I.zones[1].initted = true;
    I.zone_begin(1);
    I.zone_end();
    I.update(true);
    report := I.create_report();

    expect_equal(report.records.count, 1);
    expect_equal(report.records[0].calls, 1.0);
    r: Recorder;

    start_capture(*r);
    I.zone_begin(1);
    I.zone_end();
    I.update(true);
    stop_capture(*r);
    defer free_trace(*r.trace);
    report = I.create_report();
    expect_equal(report.records.count, 1);
    expect_equal(report.records[0].calls, 1.0);
    expect_equal(r.trace.frames.count, 1);
    expect_equal(r.trace.frames[0].spans.count, 1);
} @test @skip_on(.WASM)

empty_frames_round_trip :: (t: *Test) {
    trace: Trace;

    array_add(*trace.frames, Frame.{});
    defer free_trace(*trace);
    path := tprint("%/empty.jipt", temp_dir(t));

    expect(save_trace(*trace, path));
    loaded, ok := load_trace(path);

    defer free_trace(*loaded);
    expect(ok);
    expect_equal(loaded.names.count, 0);
    expect_equal(loaded.frames.count, 1);
    expect_equal(loaded.frames[0].spans.count, 0);
    expect_equal(loaded.frames[0].duration, cast(u64) 0);
} @test
