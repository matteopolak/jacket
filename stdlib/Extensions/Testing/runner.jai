// Running tests. The program `jacket test` builds starts in `run_tests` with the table of every
// `@test` the Discovery plugin found.
//
// Command line of a test program:
//     (nothing)              run every test here, one after another, and print a summary
//     --list                 print `name<TAB>file<TAB>line` per test
//     --run NAME...          run these tests (`test` or `test/subtest`) and report to jacket
//     --temp DIR             with --run: the I-th test's directory is DIR/I
//     --accept               snapshots that differ or are new replace the stored ones
//     --seed N               the seed property tests start from
//
// With `--run`, results go to jacket as records on standard error: `\x02` kind, fields after
// `\x1f`, then `\x03`. A `start` record is also written to standard output, so jacket can tell
// whose output follows. docs/tools/jacket-test.md lists the records.

Test_Case :: struct {
    name: string;
    body: (t: *Test);
    location: Source_Code_Location;
    flags: Test_Flags;
}

Test_Flags :: enum_flags u32 {
    ALLOW_LEAKS;
}

run_tests :: (cases: [] Test_Case) {
    runner_allocator = context.allocator;
    arguments := get_command_line_arguments();
    names: [..] string;
    temp_root := "";
    listing := false;
    i := 1;

    while i < arguments.count {
        argument := arguments[i];
        if argument == "--list" {
            listing = true;
        } else if argument == "--temp" && i + 1 < arguments.count {
            i += 1;
            temp_root = arguments[i];
        } else if argument == "--accept" {
            accepting = true;
        } else if argument == "--seed" && i + 1 < arguments.count {
            i += 1;
            seed_text := arguments[i];
            value, ok := string_to_int(seed_text, 10, u64);

            if !ok {
                print("--seed takes a whole number, not '%'\n", seed_text, to_standard_error = true);
                exit(2);
            }

            run_seed = value;
            seed_given = true;
        } else if argument == "--run" {
            reporting = true;
        } else if reporting {
            array_add(*names, argument);
        } else {
            print("unknown argument: %\nusage: [--list] [--accept] [--seed N] [--run NAME... [--temp DIR]]\n", argument, to_standard_error = true);
            exit(2);
        }

        i += 1;
    }

    if listing {
        for cases print("%\t%\t%\n", it.name, it.location.fully_pathed_filename, it.location.line_number);
        return;
    }

    if !reporting {
        run_everything(cases);
        return;
    }

    for name, index: names {
        top, rest := split_path(name);
        found: *Test_Case;

        for *cases if it.name == top found = it;

        if !found {
            record("missing", name);
            continue;
        }

        requested = rest;
        temp := ifx temp_root then tprint("%/%", temp_root, index) else "";

        run_case(found, temp);
    }
}

// Runs `body` as a test that prints nothing of its own: a failed check prints its message, and
// the program exits with status 1. `jacket test` runs a doc comment's example with it.
run_example :: (body: (t: *Test)) {
    runner_allocator = context.allocator;
    quiet = true;
    example := Test_Case.{ name = "example", body = body, flags = .ALLOW_LEAKS };
    test := run_case(*example, "");

    if test.failed exit(1);
}

#scope_module

// The seed property tests start from: `--seed`, else one drawn from the clock.
run_seed: u64;
seed_given := false;
// Whether the runner's own lines (`test ... ok`, the summary) are left out.
quiet := false;
// Whether results go to jacket as records (`--run`) or to a person (no arguments).
reporting := false;
// The subtest path asked for under the test that runs (`a/b` asks for `b` under `a`).
requested := "";
current: *Test;
// What the runner's own records (tests, subtests) come from: never a test's tracker.
runner_allocator: Allocator;
current_case: *Test_Case;
previous_assertion_handler: type_of(context.assertion_failed);

report_failure :: (loc: Source_Code_Location, message: string) {
    test := current;
    if !test {
        // A check outside a test (in a cleanup that outlived it, or in a program that never
        // started one): say so rather than lose it.
        print("%:%: % (outside a test)\n", loc.fully_pathed_filename, loc.line_number, message, to_standard_error = true);
        return;
    }

    while test {
        test.failed = true;
        test = test.parent;
    }

    if reporting {
        record("fail", current.name, loc.fully_pathed_filename, tprint("%", loc.line_number), message);
    } else if quiet {
        print("%:%: %\n", loc.fully_pathed_filename, loc.line_number, message, to_standard_error = true);
    } else {
        print("    %:%: %\n", loc.fully_pathed_filename, loc.line_number, indented(message), to_standard_error = true);
    }
}

mark_skipped :: (t: *Test, reason: string) {
    t.skipped = true;
    if reporting record("skip", t.name, reason);
}

running_location :: () -> Source_Code_Location {
    if current_case return current_case.location;
    return .{};
}

run_subtest :: (t: *Test, name: string, body: (t: *Test), body_with_data: (t: *Test, data: *void), data: *void) -> bool {
    path := tprint("%/%", t.name, name);
    if !wanted(path) return true;

    child := New(Test,, runner_allocator);
    child.name = copy_string(path,, runner_allocator);
    child.parent = t;
    outer := current;

    current = child;
    begin(child.name);
    if body body(child);
    else body_with_data(child, data);

    run_cleanups(child);
    current = outer;
    finish(child);

    return !child.failed;
}

#scope_file

split_path :: (name: string) -> (string, string) {
    for name {
        if it == #char "/" return string.{ it_index, name.data }, string.{ name.count - it_index - 1, name.data + it_index + 1 };
    }

    return name, "";
}

// Whether the subtest at `path` (`test/a/b`) runs, given what was asked for.
wanted :: (path: string) -> bool {
    if !requested return true;

    _, below := split_path(path);
    // Either one names the other or a subtest below it.
    shorter := ifx below.count < requested.count then below else requested;
    longer := ifx below.count < requested.count then requested else below;

    if string.{ shorter.count, longer.data } != shorter return false;

    return longer.count == shorter.count || longer[shorter.count] == #char "/";
}

record :: (kind: string, fields: ..string) {
    write_record(to_standard_error = true, kind, ..fields);
}

write_record :: (to_standard_error: bool, kind: string, fields: ..string) {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "\x02");
    append(*builder, kind);

    for fields {
        append(*builder, "\x1f");

        for c: cast([] u8) it {
            append(*builder, ifx c == 0x02 || c == 0x03 || c == 0x1f then #char "?" else c);
        }
    }

    append(*builder, "\x03");
    write_builder(*builder, to_standard_error = to_standard_error);
}

begin :: (name: string) {
    if !reporting return;
    record("start", name);
    // The same mark on standard output tells jacket whose output follows.
    write_record(to_standard_error = false, "start", name);
}

finish :: (t: *Test) {
    status := ifx t.failed then "fail" else ifx t.skipped then "skip" else "pass";

    if reporting {
        record("end", t.name, status);
    } else if !quiet {
        print("test % ... %\n", t.name, ifx t.failed then "FAILED" else ifx t.skipped then "skipped" else "ok");
    }
}

run_cleanups :: (t: *Test) {
    i := t.cleanups.count - 1;
    while i >= 0 {
        c := t.cleanups[i];
        c.callback(c.data);
        i -= 1;
    }

    array_reset(*t.cleanups);
}

assertion_failed :: (loc: Source_Code_Location, message: string) -> bool {
    // Alone, the usual handler's report is enough; jacket needs a record of it.
    if reporting report_failure(loc, ifx message then tprint("assertion failed: %", message) else "assertion failed");
    // The usual handler prints the stack trace and stops the program.
    return previous_assertion_handler(loc, message);
}

run_case :: (test_case: *Test_Case, temp_dir: string) -> *Test {
    test := New(Test,, runner_allocator);
    test.name = test_case.name;
    test.temp_dir = temp_dir;
    current = test;
    current_case = test_case;
    begin(test.name);
    reset_temporary_storage();
    outer := context.allocator;
    tracker := start_tracking(outer);

    previous_assertion_handler = context.assertion_failed;

    {
        new_context := context;
        new_context.allocator = tracking_allocator(tracker);
        new_context.assertion_failed = assertion_failed;
        push_context new_context {
            test_case.body(test);
            run_cleanups(test);
        }
    }

    finish_tracking(tracker, test_case.location, report_leaks = !(test_case.flags & .ALLOW_LEAKS));
    finish(test);
    current = null;
    current_case = null;
    reset_temporary_storage();

    return test;
}

run_everything :: (cases: [] Test_Case) {
    passed, failed, skipped := 0, 0, 0;

    for *cases {
        test := run_case(it, "");
        if test.failed failed += 1;
        else if test.skipped skipped += 1;
        else passed += 1;
    }

    print("\n% passed, % failed, % skipped\n", passed, failed, skipped);
    if failed exit(1);
}

indented :: (message: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for message {
        append(*builder, it);
        if it == #char "\n" append(*builder, "    ");
    }

    return builder_to_string(*builder,, temp);
}
