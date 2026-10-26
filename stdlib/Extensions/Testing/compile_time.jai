// Compile-time tests: a `@test @compile_time` procedure runs inside the compiler (a `#run` the
// Discovery plugin adds) instead of in the built program. What it reports is kept as text in
// the program, and the test the program runs replays it, so a failed check is a failed test
// and not a compile error. See docs/extensions/testing.md.

// Whether the test that is running is a `@compile_time` one, run by the compiler now.
at_compile_time :: () -> bool {
    return events != null;
}

#scope_module

// A test to run in the compiler.
Compile_Time_Case :: struct {
    name: string;
    body: (t: *Test);
}

// Where the running compile-time test writes what it reports, as separate fields; null when
// none runs.
events: *[..] string;

FIELD :: "\x1f";
GROUP :: "\x1d";

// Runs each case now and returns what they reported, one group per case: its name, its status
// (`pass`, `fail` or `skip`), then its events (`fail` file line message, `skip` reason, `log`
// text), all separated by `\x1f`, and the groups by `\x1d`.
run_compile_time :: (cases: [] Compile_Time_Case) -> string {
    builder: String_Builder;

    for cases {
        if it_index > 0 append(*builder, GROUP);

        test := New(Test);
        test.name = it.name;
        current = test;

        reported: [..] string;

        events = *reported;

        new_context := context;
        new_context.assertion_failed = assertion_in_compile_time;
        push_context new_context {
            it.body(test);
            run_cleanups(test);
        }

        events = null;
        current = null;
        status := ifx test.failed then "fail" else ifx test.skipped then "skip" else "pass";

        print_to_builder(*builder, "%1%2%3", field_text(it.name), FIELD, status);

        for reported print_to_builder(*builder, "%1%2", FIELD, it);
    }

    return builder_to_string(*builder);
}

// What a test's results say, replayed in the program that runs it: the failures, the skip and
// the output of the compile-time run.
replay_compile_time :: (t: *Test, results: string, index: s64) {
    group := nth_group(results, index);

    if !group {
        report_failure(running_location(), "this test was not run in the compiler: `jacket test` runs it when it builds the tests");
        return;
    }

    fields := split_fields(group);

    if fields.count < 2 || fields[0] != t.name {
        report_failure(running_location(), "the results of the compile-time run are not this test's");
        return;
    }

    i := 2;
    while i < fields.count {
        kind := fields[i];

        if kind == "fail" && i + 3 < fields.count {
            number, ok := string_to_int(fields[i + 2]);
            report_failure(.{ fully_pathed_filename = fields[i + 1], line_number = ifx ok then number else 0 }, fields[i + 3]);
            i += 4;
        } else if kind == "skip" && i + 1 < fields.count {
            mark_skipped(t, fields[i + 1]);
            i += 2;
        } else if kind == "log" && i + 1 < fields.count {
            print("%\n", fields[i + 1]);
            i += 2;
        } else {
            report_failure(running_location(), "the results of the compile-time run cannot be read");
            return;
        }
    }
}

// Records an event of the compile-time test that runs.
add_event :: (parts: ..string) {
    if !events return;

    for parts array_add(events, field_text(it));
}

#scope_file

// Text with the separators of the results replaced.
field_text :: (text: string) -> string {
    copy := copy_string(text,, temp);

    for *copy {
        if it.* == 0x1f || it.* == 0x1d it.* = #char "?";
    }

    return copy;
}

assertion_in_compile_time :: (loc: Source_Code_Location, message: string) -> bool {
    report_failure(loc, ifx message then tprint("assertion failed: %", message) else "assertion failed");

    // Carry on: the test is failed, and the build goes on to the next one.
    return false;
}

nth_group :: (results: string, index: s64) -> string {
    start := 0;
    found := 0;

    for results {
        if it != 0x1d continue;

        if found == index return string.{ it_index - start, results.data + start };

        found += 1;
        start = it_index + 1;
    }

    if found == index && start <= results.count && results.count > 0 return string.{ results.count - start, results.data + start };

    return "";
}

split_fields :: (group: string) -> [] string {
    fields: [..] string;
    fields.allocator = temp;
    start := 0;

    for group {
        if it != 0x1f continue;

        array_add(*fields, string.{ it_index - start, group.data + start });
        start = it_index + 1;
    }

    array_add(*fields, string.{ group.count - start, group.data + start });

    return fields;
}
