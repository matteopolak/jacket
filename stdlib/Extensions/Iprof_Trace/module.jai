// Optional, single-threaded Iprof timeline recording and a portable binary trace format.
// @experimental
// Existing Iprof reports are unchanged. Import Iprof separately in manual or plugin mode.

#import "Basic";

// One invocation, in nanoseconds relative to its frame; parent is a preceding span or -1.
Span :: struct {
    zone: s64;
    parent: s64 = -1;
    start, end: u64;
}

// A completed frame, including invocation order and nesting (recursive calls stay separate).
Frame :: struct {
    duration: u64;
    spans: [..] Span;
}

// Owned zone names and completed frames. Release with free_trace.
Trace :: struct {
    names: [..] string;
    frames: [..] Frame;
}

// Capture storage must stay at the same address until stop_capture. No concurrent profiling.
Recorder :: struct {
    trace: Trace;
    pending: Frame;
    active: [..] s64;
    origin: s64;
    last_tick: s64;
    started: bool;
}

// Attaches to Iprof before opening zones; only one recorder can be attached. Record frames with
// Iprof.update(true). Stop at a closed-zone boundary, then save and free the captured trace.
//
// ```jai,no_run
// I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);
// #import "Extensions/Iprof_Trace";
//
// main :: () {
//     recorder: Recorder;
//
//     start_capture(*recorder);
//     I.zones[0].name = "work";
//     I.zones[0].initted = true;
//     I.zone_begin(0);
//     I.zone_end();
//     I.update(true);
//     stop_capture(*recorder);
//     assert(save_trace(*recorder.trace, "work.jipt"));
//     free_trace(*recorder.trace);
// }
// ```
start_capture :: (recorder: *Recorder) {
    assert(!Observer.listener && !recorder.started && recorder.active.count == 0);
    Observer.listener_data = recorder;
    Observer.listener = receive;
}

// Detaches and drops the unfinished frame. All zones must be closed; completed frames remain.
stop_capture :: (recorder: *Recorder) {
    assert(Observer.listener_data == recorder && recorder.active.count == 0);
    Observer.listener = null;
    Observer.listener_data = null;
    array_free(recorder.pending.spans);
    array_free(recorder.active);
    recorder.pending = .{};
    recorder.active = .{};
    recorder.started = false;
}

// Releases all names, frames and spans; resets trace to an empty value.
free_trace :: (trace: *Trace) {
    for trace.names free(it);
    for *trace.frames array_free(it.spans);

    array_free(trace.names);
    array_free(trace.frames);
    trace.* = .{};
}

#load "format.jai";

#scope_file
Observer :: #import,dir "observer";

receive :: (data: *void, kind: u8, name: string, ticks: s64, seconds_per_tick: float64) {
    r := cast(*Recorder) data;
    if !r.started {
        r.origin = ticks;
        r.started = true;
    }

    assert(ticks >= r.last_tick);
    r.last_tick = ticks;
    time := cast(u64) (cast(float64) (ticks - r.origin) * seconds_per_tick * 1_000_000_000.0);

    if kind == 0 {
        found, zone := array_find(r.trace.names, name);

        if !found {
            zone = r.trace.names.count;
            array_add(*r.trace.names, copy_string(name));
        }

        parent := ifx r.active.count > 0 then r.active[r.active.count - 1] else -1;
        index := r.pending.spans.count;

        array_add(*r.pending.spans, .{ zone = zone, parent = parent, start = time });
        array_add(*r.active, index);
    } else if kind == 1 {
        assert(r.active.count > 0);
        r.pending.spans[r.active[r.active.count - 1]].end = time;
        r.active.count -= 1;
    } else {
        for r.active r.pending.spans[it].end = time;

        r.pending.duration = time;
        previous := r.pending;

        array_add(*r.trace.frames, previous);
        r.pending = .{};
        for *r.active {
            old := previous.spans[it.*];
            index := r.pending.spans.count;
            parent := ifx it_index > 0 then it_index - 1 else -1;

            array_add(*r.pending.spans, .{ zone = old.zone, parent = parent });
            it.* = index;
        }

        r.origin = ticks;
    }
}
