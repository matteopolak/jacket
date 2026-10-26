# Testing (`Extensions/Testing`)

## What it is

Unit tests for Jai code: procedures with a `@test` note, checks that report and carry on (`expect*`) or report and return (`require*`), subtests, and a per-test allocator that fails a test on a leak or a bad free. [`jacket test`](../tools/jacket-test.md) finds the tests, builds them once and runs each in its own process.

```jai
#import "Basic";
#import "Extensions/Testing";

parses_numbers :: () {
    value, ok := string_to_int("42");
    require(ok);
    expect_equal(value, 42);
} @test

splits_on_commas :: (t: *Test) {
    parts := split("a,b,c", ",",, temp);
    expect_equal(parts, string.["a", "b", "c"]);
    log(t, "parts: %", parts);
} @test @timeout(5)
```

A test is a file-level procedure that takes nothing or a `*Test` (or the arguments of its `@cases`, below), with `@test` after its body. More notes after it set options for `jacket test` (`@timeout(5)`, `@only(.LINUX,.MACOS)`; the list is in [jacket test](../tools/jacket-test.md#notes)). Notes cannot contain spaces.

## How it works

**Checks** (`checks.jai`). Each takes `loc := #caller_location`, reports at the line it is written on and returns whether it held:

| Check | Fails when |
| --- | --- |
| `expect(condition, message = "")` | `condition` is false |
| `expect_equal(got, want)` | the values differ (below) |
| `expect_near(got, want, tolerance)` | `|got - want| > tolerance`, or either is NaN |
| `expect_null(pointer)`, `expect_not_null(pointer)` | the pointer is (not) null |
| `assert_that(value, matcher)`, `require_that` | the matcher does not hold; see [Matchers](../tools/jacket-test.md#matchers) |
| `expect_snapshot(t, value, name = "")` | the printed form of `value` differs from its stored snapshot, or has none ([Snapshots](#snapshots)) |
| `require`, `require_equal`, `require_near`, `require_not_null` | as the `expect` form, then `return` from the procedure they are written in (they are macros) |

`expect_equal` compares by value (`compare.jai`): integers and enums of any width and signedness by their numeric value, floats exactly, strings by bytes, arrays of every kind element by element (a `[..] int` equals an `int.[...]` literal), structs member by member, pointers by address. A failure message says what differs:

```
got:  "abXd"
want: "abcd"
         ^ first difference at byte 2
```

Strings with line breaks get a line diff (`--- want` / `+++ got`), arrays the differing indexes plus `extra` and `missing` elements, structs the path of the first differing member (`first difference at .pos.y`), and values of unrelated types say both types.

**`*Test`** (`module.jai`):

| Procedure | Does |
| --- | --- |
| `log(t, format, ..args)` | prints a line, shown when the test fails (or with `--no-capture`) |
| `skip(t, reason = "")` | ends the test as skipped (a macro: returns from the caller) |
| `fail(t, message = "failed")` | fails the test and lets it go on |
| `temp_dir(t)` | a directory for the test's files, created by `jacket test`, deleted after a pass |
| `cleanup(t, callback, data)` | calls `callback(data)` when the test (or subtest) ends; the last registered runs first |
| `run(t, name, body)` | runs `body` as the subtest `test/name`; returns whether it passed |
| `run(t, name, body, *data)` | the same, passing `data` on, for table tests |

```jai
Case :: struct { input: string; want: int; }
CASES :: Case.[ .{ "1", 1 }, .{ "-7", -7 } ];

parses_each_case :: (t: *Test) {
    for CASES run(t, it.input, (t: *Test, c: *Case) {
        value := string_to_int(c.input);
        expect_equal(value, c.want);
    }, *it);
} @test
```

A failed subtest fails its test. `jacket test parses_each_case/-7` runs one subtest.

**Tests with cases** (`cases.jai`). `@cases(LIST)` runs a test once per entry of a list (a constant or any expression, with no spaces), each entry a subtest named from its values. A test with one argument gets the entry; a test with several gets the members of a struct entry, in order. A leading `*Test` is still the subtest's.

```jai
Parse :: struct { text: string; want: s64; }
PARSE :: Parse.[.{ "1", 1 }, .{ "-7", -7 }, .{ "+0", 0 }];

parses :: (text: string, want: s64) {
    value := string_to_int(text);
    expect_equal(value, want);
} @test @cases(PARSE)

fits_a_byte :: (n: s64) {
    expect(n >= 0 && n < 256);
} @test @cases(s64.[0,1,255])

shades :: (color: Color) {
    expect(shade(color) > 0);
} @test @cases(all_values(Color))
```

`jacket test parses` reports `parses/1,1`, `parses/-7,-7` and `parses/+0,0` as subtests; `--exact 'parses/-7,-7'` runs one. Names are the values, joined with `,` (strings as they are, `""` when empty), `/` written `|`, cut to 60 bytes, and a repeated name gets ` #N` (N: the entry's position). `all_values(T)` is the list of an enum's members in declaration order, in temporary storage. Notes such as `@xfail` and `@skip_on` apply to the whole test; inside a case, `skip(t, ...)` and `require` end that case only. A failed case fails the test but the other cases still run.

The list has to be reachable from file level (not under `#scope_file`) and a note cannot hold a space or a `{`, so give a table of structs a name (`@cases(PARSE)`) and write short lists of plain values in place. `run_cases(t, list, body)` does the same by hand, for lists built at run time; `case_names(list, body)` gives the names it would use.

**Snapshots** (`snapshots.jai`). `expect_snapshot(t, value)` compares the printed form of a value with a file kept next to the test, which is how to check a large or awkward result (a printer's output, a parsed tree) without writing it out in the test.

```jai
Point :: struct { x, y: s32; }

prints_a_point :: (t: *Test) {
    expect_snapshot(t, Point.{ 1, 2 });
    expect_snapshot(t, render(page), "page");
} @test
```

The first time, the snapshot does not exist: the test fails and the printed form is written as `NAME.snap.new` beside where the snapshot goes. `jacket test --accept` keeps new and changed snapshots (`--exact prints_a_point` to keep one test's), and `jacket test --review` shows each `.snap.new` as a diff and asks whether to accept it. A snapshot that matches again deletes its stale `.snap.new`.

| What | Where |
| --- | --- |
| a test | `tests/__snapshots__/<file>/<test>.snap` (`<file>`: the test file's name without `.jai`) |
| a case of `@cases` or a subtest | `tests/__snapshots__/<file>/<test>/<case>.snap`, one file each |
| the second snapshot of a test | `<test>.2.snap`, the third `<test>.3.snap`... |
| a snapshot with a `name` | `<test>.<name>.snap` |

A name that is not plain (`big/ger`, `what?`) has its other characters written `_` and a short hash added, so the file is legal on every OS. Strings are compared as they are; any other value is printed with one struct member or array element to a line (short arrays of numbers stay on one line), so a change shows as a line diff. Line endings and line breaks at the end of the text do not matter, so a snapshot checked out on Windows still matches. Pointers print as `null` or `<pointer>` since an address is not stable; keep them out of snapshotted values. Commit the `.snap` files; `*.snap.new` belongs in `.gitignore`.

**Memory** (`tracking.jai`). Each test runs with a tracking allocator as `context.allocator`, over the allocator that was there. It records every live block with the first stack frames that made it; when the test returns, anything still live is a failure at the frame that allocated it, outside the standard library (`leaked 32 bytes in 1 allocation made here in make_list`). Freeing or resizing a block it does not own (freed twice, or allocated elsewhere) fails the test at the first one. `@allow_leaks` turns leak reports off for a test. Temporary storage is reset before each test, so `temp` memory is never a leak. Memory allocated with another allocator is not tracked.

**Doc tests.** `jacket test` also runs the ```` ```jai ```` examples in a library's doc comments, each through this module, so `expect` and `assert` work in them (see [jacket test](../tools/jacket-test.md#doc-tests)).

**Crashes.** A failed `assert`, a crash or an `exit` ends the test's process; `jacket test` reports the test from what it wrote so far and starts a fresh process for the tests after it.

**The test program.** The Discovery plugin (`Discovery/module.jai`, run as `+Extensions/Testing/Discovery`) collects the `@test` and `@bench` procedures of the main program after type checking, writes a manifest for `jacket test` and adds a table of the tests plus an entry point, `__jacket_test_main`, that calls `run_tests` (`runner.jai`). The program's own `main` is not called. Run by hand, the program runs every test and prints a summary; with `--list` it prints the tests; with `--run NAME...` it reports to `jacket test` (the protocol is in [jacket test](../tools/jacket-test.md#the-test-program-protocol)).

## How to change it

- A new check goes in `checks.jai`, built on `report_failure(loc, message)`; add a `require_` form if it can fail, and a test in `tests/checks.jai` with `@should_fail(word)` for its message.
- How a value is printed for a snapshot, and where its file goes: `write_value` and `snapshot_path` in `snapshots.jai`. Changing either invalidates stored snapshots.
- Comparison and messages for a new kind of value: `values_equal` and `describe_difference` in `compare.jai`.
- What the tracker records (more frames, sizes): `SITE_FRAMES` and `remember` in `tracking.jai`. The tracker is never freed, because memory from a test can outlive it.
- How a case is named or an entry turned into arguments: `case_names`, `label_source` and `call_source` in `cases.jai`. They run at compile time inside `#insert`, from the procedure's and the entry's type information. `jacket test --list` learns the cases from a `#run` Discovery adds (`-cases FILE`), which calls `list_cases`.
- A new `*Test` procedure: `module.jai`. A new note is read by `jacket test` (`crates/jacket/src/test/options.rs`), except `@allow_leaks`, which Discovery also turns into a `Test_Flags` bit because the program needs it.
- A new record kind: `record` in `runner.jai`, `Demux` and `State` in `crates/jacket/src/test/exec.rs`, and the table in [jacket test](../tools/jacket-test.md#the-test-program-protocol).
- The module's own tests are in `stdlib/Extensions/Testing/tests`; run `jacket test` there (natively, with `--interp`, and for wasm).

## Configuration

None in the module. Notes on each test, and `[test]` in `jacket.toml`, are read by [`jacket test`](../tools/jacket-test.md#configuration).

## Dependencies

`Basic`, `Compiler` (Discovery), `File` and `String` (Discovery), and the compiler's stack trace information for allocation sites. Discovery is a metaprogram plugin: it needs jaic's plugin support and `add_build_string`.
