// Tests are procedures with a `@test` note, found at compile time and run by
// `jacket test`. A jaic extension, not official Jai. See docs/extensions/testing.md.
//
//     #import "Basic";
#import "File";
//     #import "Extensions/Testing";
//
//     parses_numbers :: (t: *Test) {
//         value, ok := string_to_int("42");
//         require(ok);
//         expect_equal(value, 42);
//     } @test
//
// Checks (`expect*`) report a failure and let the test go on; `require*` report it and return
// from the procedure they are written in. A test runs with a context allocator that fails it on
// a leak or a bad free, and with temporary storage reset first.

// The test that is running. `name` is `parent/child` inside a subtest.
Test :: struct {
    name: string;
    failed: bool;
    skipped: bool;

    // Private to the module.
    parent: *Test;
    cleanups: [..] Cleanup;
    temp_dir: string;
    snapshots: s64;
}

// The benchmark that is running (`bench.jai`). A `@bench` procedure loops `iterations` times.
Bench :: struct {
    iterations: s64;
    bytes_per_iteration: s64;
    items_per_iteration: s64;

    // Private to the module.
    started: Apollo_Time;
    counter: *Counter;
}

// Prints a line to the test's output, which `jacket test` shows when the test fails.
//
// ```jai,no_run
// reports_progress :: (t: *Test) {
//     log(t, "step % of %", 1, 3);
// }
// ```
//
// - t: the running test.
// - format: the format string, as for `tprint`.
// - args: the values to print into it.
log :: (t: *Test, format: string, args: ..Any) {
    text := tprint(format, ..args);

    if events {
        add_event("log", text);
        return;
    }

    print("%\n", text);
}

// Ends the test as skipped, from the procedure it is written in.
//
// - t: the running test.
// - reason: why it is skipped; shown in the report.
skip :: (t: *Test, reason := "") #expand {
    mark_skipped(t, reason);
    `return;
}

// Marks the test as failed and lets it go on.
//
// - t: the running test.
// - message: what to report.
// - loc: where the check is written, shown when it fails. Leave the default.
fail :: (t: *Test, message := "failed", loc := #caller_location) {
    report_failure(loc, message);
}

// A directory for this test's files. `jacket test` creates it and deletes it after a pass
// (`--keep-temp` keeps it); subtests share their test's directory.
//
// - t: the running test, or any of its subtests.
// - loc: where the call is written, shown if the test was given no directory. Leave the default.
temp_dir :: (t: *Test, loc := #caller_location) -> string {
    root := t;
    while root.parent root = root.parent;
    if !root.temp_dir report_failure(loc, "temp_dir: no directory was given to this test; run it with `jacket test`");
    return root.temp_dir;
}

// Calls `callback(data)` when the test ends, after its body; the last registered runs first.
//
// - t: the running test.
// - callback: called with `data` when the test ends.
// - data: passed to `callback`.
cleanup :: (t: *Test, callback: (data: *void), data: *void = null) {
    array_add(*t.cleanups, .{ callback, data });
}

// Runs `body` as the subtest `name`, reported and filtered as `test/name`. Returns whether it
// passed.
//
// - t: the parent test.
// - name: the subtest's name.
// - body: the subtest.
run :: (t: *Test, name: string, body: (t: *Test)) -> bool {
    return run_subtest(t, name, body, null, null);
}

// `run` with `data` passed on to `body`, for table tests:
//
// ```jai,ignore
// for CASES run(t, it.name, check_case, *it);
// ```
//
// - t: the parent test.
// - name: the subtest's name.
// - body: the subtest, called with `data`.
// - data: handed to `body`.
run :: (t: *Test, name: string, body: (t: *Test, data: *$T), data: *T) -> bool {
    return run_subtest(t, name, null, cast((t: *Test, data: *void)) body, data);
}

#load "checks.jai";
#load "cases.jai";
#load "matchers.jai";
#load "snapshots.jai";
#load "property.jai";
#load "shrink.jai";
#load "compile_time.jai";
#load "compare.jai";
#load "tracking.jai";
#load "runner.jai";
#load "bench.jai";

#scope_module

Cleanup :: struct {
    callback: (data: *void);
    data: *void;
}

#import "Basic";
#import "File";
