// Testing: tests are procedures with a `@test` note, found at compile time and run by
// `jacket test`. A jaic extension, not official Jai. See docs/extensions/testing.md.
//
//     #import "Basic";
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
}

// Reserved for `@bench` procedures, which `jacket test --list` lists; they do not run yet.
Bench :: struct {
    iterations: s64;
}

// Prints a line to the test's output, which `jacket test` shows when the test fails.
//
// ```jai,no_run
// reports_progress :: (t: *Test) {
//     log(t, "step % of %", 1, 3);
// }
// ```
log :: (t: *Test, format: string, args: ..Any) {
    text := tprint(format, ..args);
    print("%\n", text);
}

// Ends the test as skipped, from the procedure it is written in.
skip :: (t: *Test, reason := "") #expand {
    mark_skipped(t, reason);
    `return;
}

// Marks the test as failed and lets it go on.
fail :: (t: *Test, message := "failed", loc := #caller_location) {
    report_failure(loc, message);
}

// A directory for this test's files. `jacket test` creates it and deletes it after a pass
// (`--keep-temp` keeps it); subtests share their test's directory.
temp_dir :: (t: *Test, loc := #caller_location) -> string {
    root := t;
    while root.parent root = root.parent;
    if !root.temp_dir report_failure(loc, "temp_dir: no directory was given to this test; run it with `jacket test`");
    return root.temp_dir;
}

// Calls `callback(data)` when the test ends, after its body; the last registered runs first.
cleanup :: (t: *Test, callback: (data: *void), data: *void = null) {
    array_add(*t.cleanups, .{ callback, data });
}

// Runs `body` as the subtest `name`, reported and filtered as `test/name`. Returns whether it
// passed.
run :: (t: *Test, name: string, body: (t: *Test)) -> bool {
    return run_subtest(t, name, body, null, null);
}

// `run` with `data` passed on to `body`, for table tests:
//
//     for CASES run(t, it.name, check_case, *it);
run :: (t: *Test, name: string, body: (t: *Test, data: *$T), data: *T) -> bool {
    return run_subtest(t, name, null, cast((t: *Test, data: *void)) body, data);
}

#load "checks.jai";
#load "matchers.jai";
#load "compare.jai";
#load "tracking.jai";
#load "runner.jai";

#scope_module

Cleanup :: struct {
    callback: (data: *void);
    data: *void;
}

#import "Basic";
