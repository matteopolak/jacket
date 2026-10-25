# `jacket test`

## What it is

The test runner for Jai projects. It builds a folder's tests once, runs each test in its own process (in parallel), and reports the results as text, JSON lines or JUnit XML. Tests are written with the [`Testing` module](../extensions/testing.md); golden-output and compile-fail files need no module at all.

```
$ jacket test
    Starting 12 tests
        PASS [   0.004s] parses_numbers
        FAIL [   0.003s] splits_on_commas
        SKIP             reads_the_registry (only on windows)
...
failures:

--- splits_on_commas (FAIL) ---
  tests/strings.jai:14:
    got:  ["a", "b"]
    want: ["a", "b", "c"]
    14 | expect_equal(parts, string.["a", "b", "c"]);
  rerun: jacket test --exact 'splits_on_commas'

     Summary [   0.120s] 11 tests run: 10 passed, 1 failed, 1 skipped
```

Exit status: 0 when every test passed (or was skipped, or failed under `@xfail`), 1 when a test failed, 2 when no test matched, the tests did not compile, or the command line or `jacket.toml` was wrong.

## How it works

### Where tests are

A test root is a folder with a `tests/` directory (`crates/jacket/src/test/discover.rs`):

| File | Kind |
| --- | --- |
| `tests/name.jai` with `name.stdout`, `name.stderr` or `name.exit` beside it | golden test: built and run alone, output compared |
| `tests/compile_fail/name.jai` (with `name.stderr`) | compile-fail test: must fail to type-check with exactly those diagnostics |
| every other `tests/*.jai` | unit test file: all of them are `#load`ed into one program, whose `@test` procedures are the tests |
| a ```` ```jai ```` block in a doc comment of the library (a root with `module.jai`) | doc test: built and run as a program of its own |

Folders below `tests/` are for files the tests `#load` or `#import`. When the root holds a `module.jai`, its parent folder is added as an import folder, so the tests `#import "Name"` the module they test from the working copy.

The roots are `--path DIR` folders if given, else `[test] paths` in `jacket.toml` (only those the current directory is inside, when it is inside some), else the project's folder. With several roots, test names get the root's folder name as a prefix (`Bit_Array::finds_the_first_set`).

### Building

Per root, `jacket` writes `tests.jai` (one `#load` per unit test file) into a work folder under the system temp directory and compiles it with the Testing module's Discovery plugin:

```
jaic tests.jai -quiet [-os/-cpu/flags] [-import_dir ROOT_PARENT] -exe tests \
    +Extensions/Testing/Discovery -manifest manifest.tsv -registry registry.jai
```

The plugin writes `manifest.tsv` (`kind<TAB>name<TAB>file<TAB>line<TAB>notes...` per `@test` and `@bench`) and adds the test table and entry point. Roots build in parallel (at most four at once). A root that does not compile stops the run before any test starts.

Plugins do not run under `-interp`, and compile-time code of a wasm target runs in jaic's sandbox, whose file writes stay in memory. So with `--interp`, and for `-os wasm`, the plugin runs in a `-check` for the host and writes the code it adds to `registry.jai`; a second root file, `program.jai`, loads the tests and the registry and names the entry point in a `#run`. Workers run it with `jaic program.jai -interp`, or build it for wasm. A consequence: under `-os wasm` tests are found as the host sees them, so keep platform-specific tests apart with `@only`/`@skip_on`, not `#if OS`.

### Running

Each unit test runs as `tests --run NAME --temp DIR` in its own process, `-j` at a time (default: the machine's threads). Golden and compile-fail tests are jobs of their own. `@slow` tests start first; `@serial` tests run after the rest, one at a time.

With `--interp` or `--no-isolation`, a root's tests are split into `-j` batches and each batch runs in one process, because every `-interp` process compiles the program again. A crash or `exit` in a batch fails that test and a new process starts at the next one.

The test's own output on both streams is captured and shown with its failure; `--no-capture` streams it instead and runs one test at a time. A test that runs past its timeout has its process group killed (`kill -KILL -PID`, `taskkill /T` on Windows) and is reported `TIMEOUT`; one still running after the `slow` time gets a `SLOW` line. `--quiet` (`-q`) prints only the tests that fail, time out or are flaky, then the summary; CI uses it.

Each test gets a fresh `temp_dir` under the work folder. Folders of passing tests are deleted; a failing test's is kept and printed (`--keep-temp` keeps all).

**Status.** `PASS`, `FAIL`, `SKIP`, `TIMEOUT`; `XFAIL` (failed under `@xfail`, as expected) and `XPASS` (passed under `@xfail`, which fails the run so the note gets removed); `FLAKY` (failed, then passed on a `--retries` attempt; a pass unless `flaky_fails` is set). A test the program never reached (a crash before it) is retried in the next process; `missing` records and early exits fail it with the reason.

`--fail-fast` starts no new tests after the first failure; those left are counted as not run.

### Golden and compile-fail tests

A golden test is built (or run with `-interp`) with the same flags as the unit tests, from the folder `jacket test` was started in. It passes when its exit status equals `name.exit` (default 0), its standard output equals `name.stdout` when that file exists, and its standard error equals `name.stderr` when that file exists. Line endings are compared as `\n`; differences are shown as a line diff.

A compile-fail test is type-checked with `jaic name.jai -check -quiet -no_color` from its own folder. It passes when the check fails and the diagnostics equal `name.stderr`, with the folder written `$DIR` and the standard library `$STDLIB`:

```
bad.jai:3:12: error: type mismatch: expected `int`, got `string`
...
```

`--accept` writes what was printed into the expected files instead of comparing: for a golden test, `.stdout` (created if missing) and the `.stderr`/`.exit` files that exist; for a compile-fail test, `.stderr`.

### Doc tests

The examples in a library's doc comments are tests too (`crates/jacket/src/test/doctest.rs`). A doc comment is the run of `//` lines (`///` too) directly above a declaration, or the run that opens `module.jai`, which documents the module. jacket reads the root's `.jai` files outside `tests/` and outside folders that hold a `module.jai` of their own, then every file they `#load`, wherever it lives, and the `internal/...` modules they `#import` (most standard library modules keep their code in `stdlib/internal`). A block in a loaded file belongs to the root that loads it; a file two roots load is read for the first one only. Only the root's own `module.jai` gives module docs.

```jai
// The pieces of `s` between each `separator`.
//
// ```jai
// parts := split("a,b,c", ",");
// assert(parts.count == 3 && parts[1] == "b");
// ```
split :: Impl.split;
```

Each block is a program that imports `Basic` and the library (by its path in the standard library, `Extensions/Testing`, or else by its folder's name), unless the block imports them itself. A block with a file-level `main` is used as it is. Otherwise its lines become the body of a procedure, with its `#import` and `#load` lines moved to file level. Either way the example runs through the Testing module with leak checks off, so a failed `assert`, `expect` or `require` fails the doc test. Diagnostics and stack traces name the doc comment's file and line, and a failure shows only the example's own output, without the runner's lines.

Only fences that start with `jai` are tests. Options follow it, separated by commas or spaces:

| Fence | The doc test |
| --- | --- |
| ```` ```jai ```` | is built and run, and must exit with status 0 |
| ```` ```jai,no_run ```` | is only type-checked |
| ```` ```jai,compile_fail ```` | must fail to type-check |
| ```` ```jai,should_fail ```` | is built and run, and must fail: a failed check, an assert, a crash or a nonzero exit |
| ```` ```jai,ignore ```` | is skipped |

The notes a `@test` takes are options too, without the `@`: ```` ```jai,only(linux, macos) ````, ```` ```jai,skip_on(wasm) ````, `needs(...)` and `timeout(...)` work as on a unit test (below).

An unknown option is a warning. Doc tests are named `doc: Module.symbol#N`, or `doc: Module#N` for the module's own docs, where N counts the symbol's blocks from 1. They carry the file and line of their fence. `--doc` runs only doc tests and `--no-doc` leaves them out. `no_run` and `compile_fail` blocks run for any `-os`/`-cpu`; the others need `--runner` on another target, as unit tests do.

### Filters

`jacket test parse split` runs the tests whose name contains `parse` or `split`; `--exact` makes filters whole names. `name/sub` runs one subtest of `name` (subtests are reported under their test). `--partition hash:K/N` keeps the tests whose name hashes (FNV-1a) to part K of N, stable across machines; `count:K/N` takes every Nth by position. `--list` prints `name: kind` per test without running anything (`--format json`: one `{"type":"list",...}` per test).

### Notes

Options for one test are notes after its body; a golden or compile-fail file, or every test of a unit test file, takes the same options from `// test:` lines in the comment block at the top of the file:

```jai
// test: only(.LINUX,.MACOS) timeout(10)
#import "Extensions/Testing";

reads_proc_self :: () { ... } @test @serial
```

| Note | Effect |
| --- | --- |
| `@timeout(N)` | kill after N seconds (0: no limit); default `--timeout`, else `[test] timeout`, else 120 |
| `@slow` | known to be slow: started first, no `SLOW` line |
| `@serial` | runs while no other test does |
| `@retries(N)` | retries for this test, over `--retries` |
| `@only(...)` | run only on these targets: `.LINUX`, `.MACOS`, `.WINDOWS`, `.WASM`, `.X64`, `.ARM64`, `.INTERP`, `.NATIVE`; every kind named must match (`only(.LINUX,.X64)`) |
| `@skip_on(...)` | skip on any of these targets |
| `@needs(.NETWORK)` | skip unless the run has the capability (`--needs network`, `[test] needs`); any name works |
| `@interp_only`, `@native_only` | run only with (without) `--interp` |
| `@exit(N)` | the program must exit with status N during the test, with no failed check |
| `@should_fail`, `@should_fail(word)` | must fail (a check, a crash or a timeout); with a word, a failure message or the test's standard error must contain it |
| `@xfail`, `@xfail(reason)` | a known failure, reported `XFAIL`; passing is an error |
| `@allow_leaks` | the tracking allocator reports no leaks |

An unknown note or a bad argument is a warning naming the file and line; the note is ignored. The target is the host, or `-os`/`-cpu`; a cross-built test is skipped unless `--runner` can run it (compile-fail tests still run).

### Targets

```
jacket test --interp                                   # in jaic's interpreter
jacket test -os wasm --interp                          # in jaic's wasm sandbox
jacket test -os wasm --runner "node tools/wasi_run.mjs" # a wasm build under node
jacket test -- -O2                                     # flags after `--` go to jaic
```

`-os` and `-cpu` are spelled as in jaic. `--runner` is split on spaces, and the built program and its arguments follow its words; it applies to unit and golden tests. Tests run in their own folders, so a word naming an existing file is made absolute: from the current directory for `--runner`, from the folder of `jacket.toml` for `[test] runner`. Flags after `--` that only a build takes (`-O2`, `-exe`, ...) are left out where jacket checks or interprets instead (listing, compile-fail tests, `--interp`).

### Output formats

`--format json` writes one JSON object per line to standard output, as things happen:

| `type`, `event` | Fields |
| --- | --- |
| `run`, `started` | `tests` |
| `test`, `started` | `name`, `kind` |
| `test`, `slow` | `name`, `seconds` |
| `test`, `passed` / `failed` / `skipped` / `timed_out` / `xfail` / `xpass` / `flaky` | `name`, `kind`, `file`, `line`, `duration` (seconds), `attempts`, `failures` (`test`, `file`, `line`, `message`), `problems` (strings: diffs, exits, timeouts), `subtests` (`name`, `status`, `duration`), `stdout`, `stderr`, `reason` (skips), `temp_dir` (kept folder or null) |
| `run`, `finished` | `passed`, `failed`, `skipped`, `timed_out`, `xfail`, `xpass`, `flaky`, `not_run`, `duration` |
| `list` | `name`, `kind`, `file`, `line`, `notes` (with `--list`) |

`kind` is `test`, `golden`, `compile_fail`, `doc` or `bench`. `type: "bench"` is reserved for benchmark results. Warnings and build errors stay on standard error.

`--junit FILE` also writes JUnit XML: a `<testsuite>` per root, a `<testcase>` per test with `<failure>`, `<skipped>` and the captured output.

### The test program protocol

The program Discovery builds (`stdlib/Extensions/Testing/runner.jai`) takes:

```
tests                       run every test here and print a summary (for running by hand)
tests --list                name<TAB>file<TAB>line per test
tests --run NAME... [--temp DIR]   run these tests, the I-th with temp folder DIR/I, and report
```

With `--run` it writes records to standard error: `\x02`, the kind, each field after `\x1f`, then `\x03` (field bytes `\x02`, `\x03`, `\x1f` are written `?`). The `start` record also goes to standard output, so `jacket` knows which test the output that follows belongs to.

| Record | Fields | Meaning |
| --- | --- | --- |
| `start` | name | a test or subtest (`test/sub`) begins |
| `fail` | name, file, line, message | a check failed |
| `skip` | name, reason | the test skipped itself |
| `end` | name, `pass` / `fail` / `skip` | it finished |
| `missing` | name | no test has this name |

Everything else on either stream is the test's own output. A process that stops after `start` without `end` crashed in that test (`exec.rs` turns its exit status or signal into the reason).

### `jacket bench`

Reserved. `jacket bench --list` lists `@bench` procedures (`name :: (b: *Bench) { ... } @bench`); running them is not supported yet, and `jacket bench` without `--list` exits 2 saying so. `jacket test --list` lists them with kind `bench`.

## Design decisions

- **Notes are lowercase (`@test`)**, like the language's other notes, and go after the procedure body, where Jai puts them. There is no `test "name" { }` macro: the procedure's name is the test's name, which filters, rerun lines and the editor use. A macro would be a second spelling for the same thing.
- **jacket runs jaic as a program**, not in-process. `jacket` is a front door to separate tools and links none of them; a separate compile also means the tests build exactly as `jaic` builds them. The cost, one compile per root and run, is the same either way.
- **`--interp` isolation is per batch, not per test.** Each `-interp` process compiles the program again, so one per test would cost a compile per test. Tests in a batch still get a fresh tracking allocator, temp folder and temporary storage, and a crash only ends its batch: the next process starts at the next test.
- **Per-test options are notes on the test**; `jacket.toml` has only run-wide settings. A test's requirements move with it, and a test file is readable alone.
- **Doc tests run every ```` ```jai ```` block**, as rustdoc does with Rust's, and need no `@test`: an example that no longer compiles or runs is a failure. Bare statements are wrapped in a procedure, so a short example needs no `main`. Untagged fences are not run, since comments written before doc tests existed use them for other text.
- **Expected-output files hold whole output**, not substrings: a change in a diagnostic shows up as a diff and `--accept` updates it. Paths are written `$DIR` and `$STDLIB`.
- **The design follows** cargo-nextest (process per test, retries and flaky reports, partitions, JSON events), Go's `testing` (`*Test`, subtests, `cleanup`, `temp_dir`), Zig's testing allocator (leak and bad-free checks), pytest (`xfail`), and trybuild (compile-fail files).

## How to change it

- A new flag: `TestArgs` in `crates/jacket/src/cli.rs`, read in `settings` in `crates/jacket/src/test/mod.rs`; if it has a `jacket.toml` key, add it to `Test` and `TestRun` in `crates/jacket-config/src/schema.rs` (and the known-keys lists and the VS Code schema in `editors/vscode/scripts/build.ts`).
- A new note: `TestOptions::apply` and `skip_reason` in `crates/jacket/src/test/options.rs`, then how it is used in `jobs.rs` (`resolve`, `expectations`) or `mod.rs`; list it in the table above.
- A new kind of file test: `Root::read` (`discover.rs`), a `Kind`, and a runner in `golden.rs` called from `run_file_case` (`jobs.rs`).
- Doc tests: what counts as a doc comment, the fence options and the generated program are in `doctest.rs` (`file_blocks`, `fenced`, `generate`); checking a block is `doctest::check`.
- Scheduling and batching: `jobs` and `run_jobs` in `jobs.rs`. Process handling, record parsing and timeouts: `exec.rs`. Output: `report.rs`.
- The JSON fields are a format others read (CI, the language server): add fields freely, and change or remove one only with a changelog line.
- Unit tests for the runner are in each file; `cargo test -p jacket`. The Testing module's Jai tests run with `jacket test` in `stdlib/Extensions/Testing`.

## Configuration

```toml
[test]
paths = ["stdlib/Bit_Array", "stdlib/String"]   # test roots; default: the project folder
jobs = 8                 # -j
timeout = 120            # --timeout, seconds; 0: no limit
slow = 30                # seconds before a SLOW line
retries = 0              # --retries
fail_fast = false        # --fail-fast
flaky_fails = false      # FLAKY fails the run
runner = "node tools/wasi_run.mjs"   # --runner
needs = ["network"]      # --needs

[test.profile.ci]        # --profile ci: these over [test]
retries = 2
fail_fast = true
```

Flags win over the profile, the profile over `[test]`. Unknown keys get the usual `jacket.toml` warning. Environment: `JAIC_STDLIB` (where `$STDLIB` points, as for jaic), `TMPDIR` (the work folder).

## Dependencies

`jaic` (found as for every jacket command) and the `Extensions/Testing` module in its standard library; `serde_json` for JSON output. `--runner` programs, such as `node` for `tools/wasi_run.mjs`, are the user's.
