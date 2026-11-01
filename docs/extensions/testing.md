# Testing

Unit tests for Jai code: procedures with a `@test` note, checks that report and carry on (`expect*`) or report and return (`require*`), subtests, and a per-test allocator that fails a test on a leak or bad free. [`jacket test`](../tools/jacket-test.md) finds the tests, builds them once and runs each in its own process.

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

A test is a file-level procedure taking nothing or a `*Test` (or the arguments of its `@cases`/`@property`) with `@test` after its body. Further notes set options for `jacket test` (`@timeout(5)`, `@only(.LINUX,.MACOS)`, `@should_fail`, `@xfail`, `@allow_leaks`; see [jacket test](../tools/jacket-test.md#writing-tests)). Notes can't contain spaces.

## How it works

- **Checks** (`checks.jai`) take `loc := #caller_location`, report at the line they're written on and return whether they held: `expect`, `expect_equal`, `expect_near`, `expect_null`, `expect_not_null`, `assert_that`, `expect_snapshot`. The `require_` forms then `return` from the calling procedure (they are macros).
- `expect_equal` compares by value (`compare.jai`): numbers by value, strings by bytes, arrays and structs element by element, pointers by address. Failures show what differs: the first differing byte, a line diff for multi-line strings, differing indexes, or the path of the first differing member.
- **`*Test`** has `log` (shown on failure), `skip`, `fail`, `temp_dir`, `cleanup(callback, data)` and `run(t, name, body)` for subtests. `jacket test parses_each_case/-7` runs one subtest.
- **`@cases(LIST)`** (`cases.jai`) runs a test once per list entry as a subtest; struct entries are spread over the arguments. Names are the values joined with `,`. The list must be reachable from file level and can't contain spaces, so give a table a name. `all_values(T)` lists an enum's members. `run_cases` does the same by hand.

```jai
Parse :: struct { text: string; want: s64; }
PARSE :: Parse.[.{ "1", 1 }, .{ "-7", -7 }];

parses :: (text: string, want: s64) {
    expect_equal(string_to_int(text), want);
} @test @cases(PARSE)
```

- **Snapshots** (`snapshots.jai`). `expect_snapshot(t, value, name = "")` compares the printed form of a value with a file under `tests/__snapshots__/<file>/<test>.snap`. A missing snapshot fails the test and writes `.snap.new`; `jacket test --accept` keeps new and changed ones, `--review` shows diffs. Pointers print as `<pointer>`, so keep them out of snapshotted values. Commit `.snap` files and ignore `*.snap.new`.
- **Property tests** (`property.jai`, `shrink.jai`). `@property` runs the procedure on generated arguments (100 times; `@property(500)`). Arguments can be integers, floats, bools, strings, enums, arrays, structs of those and pointers to them. A property passes when its checks hold, or returns true if it returns a `bool`. `assume(condition)` discards a trial. A failing input is shrunk, and the report gives the smallest counterexample and a rerun line with the seed (`jacket test --seed N`); the same seed makes the same inputs in every OS and mode.

```jai
reversing_twice_is_the_identity :: (items: [] s64) {
    expect_equal(reversed(reversed(items)), items);
} @test @property
```

- **Custom generators.** `@generate(NAME)` names a procedure `(rng: *Rng) -> T` that replaces the built-in values for type `T`. `Rng` has `rng_u64`, `rng_below`, `rng_range`, `rng_chance`, `rng_unit` and `rng.size`. Custom values aren't shrunk. Properties don't combine with `@cases` and can't use subtests.
- **Compile-time tests.** `@compile_time` after `@test` runs the procedure inside the compiler, for code meant to run in a `#run`. The Discovery plugin runs them while the tests are built and replays what they reported, so failures are ordinary `FAIL`s. A crash that isn't an `assert` stops the build; there is no leak tracking; `--interp` runs them in every interpreter process.
- **Memory** (`tracking.jai`). Each test runs with a tracking allocator as `context.allocator`. Anything live at the end is a failure at the line that asked for it: when the memory is allocated, the stack is walked to the first frame outside the standard library (files under a `tests/` folder of the library count as yours), whatever the depth of the library's own calls, and that frame is stored with the allocation; freeing a block it doesn't own fails at the first one. `@allow_leaks` turns leak reports off. Temporary storage is reset per test. Memory from other allocators isn't tracked.
- **Doc tests.** `jacket test` also runs the `jai` examples in doc comments through this module ([doc tests](../tools/jacket-test.md#doc-tests)).
- **Crashes.** A failed `assert`, crash or `exit` ends the test's process; `jacket test` reports from what it wrote and starts a fresh process for the rest.
- **The test program.** The Discovery plugin (`Discovery/module.jai`, run as `+Extensions/Testing/Discovery`) collects `@test` and `@bench` procedures after type checking and adds a table plus an entry point `__jacket_test_main` that calls `run_tests` (`runner.jai`). The program's own `main` is not called. Run by hand it runs everything and prints a summary; `--list` names the cases; `--run NAME...` reports to `jacket test` (protocol in [jacket test](../tools/jacket-test.md#how-it-works)).

## How to change it

- New check: in `checks.jai` on `report_failure(loc, message)`, with a `require_` form if it can fail, and a test in `tests/checks.jai` with `@should_fail(word)` for its message.
- Printing and location of snapshots: `write_value`, `snapshot_path`. Changing either invalidates stored snapshots.
- New generated value kind: `make_value` (and `unsupported`) in `property.jai`, then `smaller_values` in `shrink.jai`. Don't change the number sequence for existing seeds without a changelog line; people keep seeds in bug reports.
- Comparison and messages: `values_equal`, `describe_difference` in `compare.jai`.
- Compile-time results are text (`run_compile_time` writes, `replay_compile_time` reads): a new event kind needs a case in both. The `#run` is generated by `registry_source` in `Discovery/module.jai`.
- Case naming and argument spreading: `case_names`, `label_source`, `call_source` in `cases.jai` (they run in `#insert`).
- New note: `crates/jacket/src/test/options.rs`, except `@allow_leaks`, which Discovery also turns into a `Test_Flags` bit. New record kind: `record` in `runner.jai`, `Demux` and `State` in `crates/jacket/src/test/exec.rs`.
- Benchmarks (`@bench`) live in `bench.jai` and run under [jacket bench](../tools/jacket-bench.md). `black_box` is a compiler intrinsic ([intrinsics](../language/intrinsics.md)).
- The module's own tests are in `stdlib/Extensions/Testing/tests`; run `jacket test` there, natively, with `--interp`, and for wasm.

Needs `Basic`, `Compiler`, `File`, `String`, and jaic's plugin support (`add_build_string`).
