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

A test is a file-level procedure that takes nothing or a `*Test` (or the arguments of its `@cases` or `@property`, below), with `@test` after its body. More notes after it set options for `jacket test` (`@timeout(5)`, `@only(.LINUX,.MACOS)`; the list is in [jacket test](../tools/jacket-test.md#notes)). Notes cannot contain spaces.

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

**Property tests** (`property.jai`, `shrink.jai`). `@property` on a procedure runs it on generated arguments, 100 times by default (`@property(500)` for another count). The arguments may be integers, floats, `bool`s, strings, enums, arrays (fixed, view and resizable), structs of those, and pointers to them. A leading `*Test` is the test's, as in any `@test`.

```jai
reversing_twice_is_the_identity :: (items: [] s64) {
    expect_equal(reversed(reversed(items)), items);
} @test @property

addition_commutes :: (a: s32, b: s32) -> bool {
    return cast(s64) a + b == cast(s64) b + a;
} @test @property(500)
```

A property passes when its checks hold on every input, or, when it returns a `bool`, when it returns true. `assume(condition)` ends a trial without failing it (a property that returns a value writes `if !condition return true;`). Early trials use small values (short strings and arrays, small integers, with 0, 1 and the extremes of the type mixed in) and later ones larger, up to 100 elements.

When an input fails, it is shrunk: integers move toward 0, floats toward 0 and whole numbers, strings and arrays get shorter and their elements simpler, struct members shrink one at a time. What is reported is the smallest input found that still fails, then the failure itself at the line of the check that made it:

```
--- lists_shrink_to_one_element (FAIL) ---
  tests/property.jai:79:
    property failed on trial 4 (seed 10254160)
    counterexample: lists_shrink_to_one_element([100])
    shrunk from:    lists_shrink_to_one_element([1358734529538984603, 5, 6, -5]) in 57 steps
    rerun: jacket test --seed 10254160 --exact 'lists_shrink_to_one_element'
  tests/property.jai:80:
    expected true, got false
    80 | for items expect(it < 100);
```

The seed comes from `jacket test --seed N`; without it each run draws one and prints it only with a failure. A run with the same seed makes the same inputs on every OS and in every mode, so the rerun line repeats the failure. A failed `assert` inside a property fails the input like a check does (the run goes on after it, as with any assertion handler that returns), so it is shrunk too.

**Custom generators.** `@generate(NAME)` names a procedure `(rng: *Rng) -> T` whose values replace the built-in ones for every argument (or struct member, or element) of type `T`. `Rng` has `rng_u64`, `rng_below(rng, n)`, `rng_range(rng, low, high)`, `rng_chance(rng, one_in)`, `rng_unit` (a float from 0 to 1) and `rng.size`, which grows from 1 to 100 over the trials. Values from a custom generator are not shrunk, since the module cannot know what the generator guarantees.

```jai
even :: (rng: *Rng) -> s64 { return rng_range(rng, -500, 500) * 2; }

doubling_keeps_parity :: (n: s64) {
    expect_equal((n * 2) % 2, 0);
} @test @property @generate(even)
```

`run_property(t, procedure, count, generators)` does the same by hand, with `Custom_Generator.[generator(even)]`. A procedure that needs a type the module cannot generate (a procedure pointer, `Any`) fails with a message naming the type and suggesting `@generate`. Properties do not combine with `@cases` and cannot use subtests.

**Compile-time tests** (`compile_time.jai`). `@compile_time` after `@test` runs the procedure inside the compiler, which is how to test code that is meant to run in a `#run`, or constants and generated code:

```jai
triangle :: (n: s64) -> s64 { sum := 0; for 1..n sum += it; return sum; }

sums_in_the_compiler :: () {
    expect_equal(triangle(10), 55);
    expect_equal(#run triangle(4), 10);
} @test @compile_time
```

The Discovery plugin adds a `#run` that calls every such test while the tests are built and keeps what they reported (failed checks with their file and line, `log` output, a skip) as text in the program. The test the program runs only replays that, so a failure is an ordinary `FAIL` with the check's message, filters, `--list`, `@should_fail`, `@xfail` and `@skip_on` work as for any test, and one failing test does not stop the build or the others. A failed `assert` in the compiler is a failed check too, and the test carries on, as `assert` does with a handler that returns.

Run-time facts hold at compile time as the compiler's interpreter has them: `at_compile_time()` is true in a test that runs there, and what needs the operating system (files, threads, time) may not work. The test cannot take a `@cases` list or be a `@property`, and a crash that is not an `assert` (a bad pointer, `exit`) stops the compiler and so the build. There is no leak tracking, because the tracker needs the program's allocator and stack frames.

They run once per build, whatever the mode: natively, under `-O2` and for `-os wasm` the `#run` is part of building the test program. With `--interp` (and `-os wasm --interp`) there is no build, so the `#run` is part of every interpreter process that runs the program (one per `-j` batch). `jacket test --list` does not run them.

**Memory** (`tracking.jai`). Each test runs with a tracking allocator as `context.allocator`, over the allocator that was there. It records every live block with the first stack frames that made it; when the test returns, anything still live is a failure at the frame that allocated it, outside the standard library (`leaked 32 bytes in 1 allocation made here in make_list`). Freeing or resizing a block it does not own (freed twice, or allocated elsewhere) fails the test at the first one. `@allow_leaks` turns leak reports off for a test. Temporary storage is reset before each test, so `temp` memory is never a leak. Memory allocated with another allocator is not tracked.

**Doc tests.** `jacket test` also runs the ```` ```jai ```` examples in a library's doc comments, each through this module, so `expect` and `assert` work in them (see [jacket test](../tools/jacket-test.md#doc-tests)).

**Crashes.** A failed `assert`, a crash or an `exit` ends the test's process; `jacket test` reports the test from what it wrote so far and starts a fresh process for the tests after it.

**The test program.** The Discovery plugin (`Discovery/module.jai`, run as `+Extensions/Testing/Discovery`) collects the `@test` and `@bench` procedures of the main program after type checking, writes a manifest for `jacket test` and adds a table of the tests plus an entry point, `__jacket_test_main`, that calls `run_tests` (`runner.jai`). The program's own `main` is not called. Run by hand, the program runs every test and prints a summary; with `--list` it prints the tests; with `--run NAME...` it reports to `jacket test` (the protocol is in [jacket test](../tools/jacket-test.md#the-test-program-protocol)).

## How to change it

- Compile-time results are text (`run_compile_time` writes it, `replay_compile_time` reads it): a new kind of event needs a case in both, and in `report_failure`/`mark_skipped`/`log`, which write them. The `#run` is generated by `registry_source` in `Discovery/module.jai`.
- A new check goes in `checks.jai`, built on `report_failure(loc, message)`; add a `require_` form if it can fail, and a test in `tests/checks.jai` with `@should_fail(word)` for its message.
- How a value is printed for a snapshot, and where its file goes: `write_value` and `snapshot_path` in `snapshots.jai`. Changing either invalidates stored snapshots.
- A new kind of generated value: `make_value` in `property.jai` (and `unsupported`, which lists what can be generated), then `smaller_values` in `shrink.jai`. The sequence of numbers must not change for existing seeds without a changelog line: people keep seeds in bug reports.
- Comparison and messages for a new kind of value: `values_equal` and `describe_difference` in `compare.jai`.
- What the tracker records (more frames, sizes): `SITE_FRAMES` and `remember` in `tracking.jai`. The tracker is never freed, because memory from a test can outlive it.
- How a case is named or an entry turned into arguments: `case_names`, `label_source` and `call_source` in `cases.jai`. They run at compile time inside `#insert`, from the procedure's and the entry's type information. `jacket test --list` learns the cases from a `#run` Discovery adds (`-cases FILE`), which calls `list_cases`.
- A new `*Test` procedure: `module.jai`. A new note is read by `jacket test` (`crates/jacket/src/test/options.rs`), except `@allow_leaks`, which Discovery also turns into a `Test_Flags` bit because the program needs it.
- A new record kind: `record` in `runner.jai`, `Demux` and `State` in `crates/jacket/src/test/exec.rs`, and the table in [jacket test](../tools/jacket-test.md#the-test-program-protocol).
- Benchmarks (`@bench`, `Bench`) live in `bench.jai` and are run by [jacket bench](../tools/jacket-bench.md), not by `run_tests`'s default mode. `black_box`, declared there, is a compiler intrinsic ([intrinsics](../language/intrinsics.md)); the sampling and statistics are in jacket (`bench_stats.rs`).
- The module's own tests are in `stdlib/Extensions/Testing/tests`; run `jacket test` there (natively, with `--interp`, and for wasm).

## Configuration

None in the module. Notes on each test, and `[test]` in `jacket.toml`, are read by [`jacket test`](../tools/jacket-test.md#configuration).

## Dependencies

`Basic`, `Compiler` (Discovery), `File` and `String` (Discovery), and the compiler's stack trace information for allocation sites. Discovery is a metaprogram plugin: it needs jaic's plugin support and `add_build_string`.
