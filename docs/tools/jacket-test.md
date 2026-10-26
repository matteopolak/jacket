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
| a `.jai` file in a folder below `tests/` with `name.stdout`, `name.stderr` or `name.exit` beside it | golden test too (named `folder/name`); with `// test: compile_fail` on top, a compile-fail test whose `name.stderr` holds the diagnostics |
| every other `tests/*.jai` | unit test file: all of them are `#load`ed into one program, whose `@test` procedures are the tests |
| a ```` ```jai ```` block in a doc comment of the library (a root with `module.jai`) | doc test: built and run as a program of its own |

The rest of the files below `tests/` are for what the tests `#load` or `#import`.

**Layouts.** A root is the folder that holds `tests/`. Two layouts are supported:

| Layout | How the tests reach the code |
| --- | --- |
| a library folder: `Name/module.jai`, `Name/tests/` | the root holds a `module.jai`, so its parent folder is added as an import folder and the tests `#import "Name"` the working copy. Only a library root has doc tests; `Name` is the folder's name (or, below the standard library, its path there) |
| a project: `tests/` beside the code, no `module.jai` in the root | the project's `[project] import_path` folders are passed as `-import_dir`, as for `jacket build`, so `import_path = ["src"]` with `src/Name/module.jai` lets the tests `#import "Name"`; they `#load` anything else by path |

A `src/module.jai` next to `tests/` has no folder to name the module and is not detected: put the module in `src/Name/` and list `src` in `import_path`. The library root is never guessed from names: it is the folder with `module.jai`, and everything else is on `import_path`.

**Roots.** They are `--path DIR` folders if given, else `[test] paths` in `jacket.toml` (only those the current directory is inside, when it is inside some), else the project's folder. `--list` prints the roots it searched to standard error (`jacket: root /path (library)`), since the current directory can narrow them. With several roots, test names get the root's folder name as a prefix (`Bit_Array::finds_the_first_set`). Paths among the flags (`--path`, `--junit`, and `-import_dir`, `-I`, `-output_path`, `-emit_ir`, `-coverage_file` after `--`) are relative to the directory `jacket test` is run from; they are made absolute once, before any test builds in its own folder.

### Building

Per root, `jacket` writes `tests.jai` (one `#load` per unit test file) into a work folder under the system temp directory and compiles it with the Testing module's Discovery plugin:

```
jaic tests.jai -quiet [-os/-cpu/flags] [-import_dir ROOT_PARENT] -exe tests +Extensions/Testing/Discovery
```

The plugin takes no options. It finds the `@test` and `@bench` procedures, adds the test table and the entry point, and prints what it found on standard output as records (`\x02`, the kind, each field after `\x1f`, then `\x03`, the framing of the [test program's records](#the-test-program-protocol)): one `manifest` record per test or bench (kind, name, file, line, then its notes), one `source` record per line of the code it added, and `discovered` last. jacket reads the records and ignores the rest of the output. One compile finds the tests and builds them, for every mode. Roots build in parallel (at most four at once). A root that does not compile stops the run before any test starts.

- **Native** and **`-os wasm`**: the build above writes the executable (`tests.wasm` for wasm) and jacket takes the manifest from its output. Compile-time code of a wasm target runs in jaic's sandbox, whose file writes stay in memory, which is why the plugin prints and never writes. Tests are found as the target sees them, so `#if OS` works.
- **`--interp`**: there is no program to write, and every `jaic -interp` process compiles the root again. A plugin makes the compiler check all of the root, about three times the work of running it (0.54 s against 0.18 s for the `stdlib/Extensions/Testing` root, debug), so only the process that runs the first tests uses it. It is started at once, as `jaic tests.jai -interp ... +Extensions/Testing/Discovery -- --args-from plan.txt`, and jacket reads its records while it compiles (a process that ends first did not compile, and its output is the build error). jacket then writes `registry.jai` from the `source` records and `program.jai`, which `#load`s the tests and `registry.jai` and names the entry point in a `#run`, and plans the run. It writes the first job's arguments to `plan.txt`, which the waiting process runs. The other processes (the rest of the `-j` batches, retries after a crash) run `program.jai`, which compiles only what the tests reach.
- **`--interp -os wasm`**: the program runs in the sandbox, which prints to jacket only when the program ends and has no `plan.txt`. So the first process is not kept: it runs `-- --list` (below) to its end, jacket reads the records, and every run is a `program.jai` process.
- **`--list`**: the same, for the host (`-os` and `-cpu` are left out): `-- --list` makes the program name the cases of each `@cases` test as `case` records and run nothing, and the build errors of a program that does not compile are reported. Nothing is built or run.
- **`--coverage`** builds natively with `-coverage`; it cannot be combined with `--interp`.

Why `--interp` keeps `program.jai` and does not start every process with the plugin, or run every test in the first process: measured over three runs on `stdlib/Extensions/Testing` and `stdlib/Hash_Table`, a plugin in every process took 0.51 s and 0.58 s in a release build against 0.36 s and 0.40 s; one process for the whole root (`-j1`) took 2.1 s and 2.7 s in a debug build against 1.3 s and 1.5 s. The processes run in parallel and the plugin's check is on the critical path of none but the first. Forking a compiled program or sharing one between processes needs support in `jaic` that does not exist.

### Running

Each unit test runs as `tests --run NAME --temp DIR` in its own process, `-j` at a time (default: the machine's threads). Golden and compile-fail tests are jobs of their own. `@slow` tests start first; `@serial` tests run after the rest, one at a time.

With `--interp` or `--no-isolation`, a root's tests are split into `-j` batches and each batch runs in one process, because every `-interp` process compiles the program again. A crash or `exit` in a batch fails that test and a new process starts at the next one.

A failed test with one to three failed subtests prints a `rerun:` line naming those subtests.

The test's own output on both streams is captured and shown with its failure; `--no-capture` streams it instead and runs one test at a time. A test that runs past its timeout has its process group killed (`kill -s KILL -- -PID`; the `--` keeps procps `kill` from reading `-PID` as an option and signalling every process, `taskkill /T` on Windows) and is reported `TIMEOUT`; one still running after the `slow` time gets a `SLOW` line. `--quiet` (`-q`) prints only the tests that fail, time out or are flaky, then the summary; CI uses it.

Each test gets a fresh `temp_dir` under the work folder. Folders of passing tests are deleted; a failing test's is kept and printed (`--keep-temp` keeps all).

**Status.** `PASS`, `FAIL`, `SKIP`, `TIMEOUT`; `XFAIL` (failed under `@xfail`, as expected) and `XPASS` (passed under `@xfail`, which fails the run so the note gets removed); `FLAKY` (failed, then passed on a `--retries` attempt; a pass unless `flaky_fails` is set). A test the program never reached (a crash before it) is retried in the next process; `missing` records and early exits fail it with the reason.

`--fail-fast` starts no new tests after the first failure; those left are counted as not run.

### Golden and compile-fail tests

A golden test is built (or run with `-interp`) with the same flags as the unit tests, from its own folder. It passes when its exit status equals `name.exit` (default 0), its standard output equals `name.stdout` when that file exists, and its standard error equals `name.stderr` when that file exists. Line endings are compared as `\n`; differences are shown as a line diff.

A golden program that has no `main` and does its work at compile time (`#run`, a metaprogram) is only type-checked, and what the compiler prints is its output. A program with `@warns(text)` is built without `-quiet` and passes when the compiler's output (interpreted: standard error) contains the text.

A compile-fail test is type-checked with `jaic name.jai -check -quiet -no_color` from its own folder. It passes when the check fails and the diagnostics equal `name.stderr`, with the folder written `$DIR` and the standard library `$STDLIB`:

```
bad.jai:3:12: error: type mismatch: expected `int`, got `string`
...
```

`--accept` writes what was printed into the expected files instead of comparing: for a golden test, `.stdout` (created if missing) and the `.stderr`/`.exit` files that exist; for a compile-fail test, `.stderr`. Unit tests get it too, as the snapshots they take (below).

### Snapshots

A unit test can compare a value with a stored snapshot (`expect_snapshot`, see [Testing](../extensions/testing.md#how-it-works)). A snapshot that differs or does not exist fails the test and is written as `NAME.snap.new` next to where `NAME.snap` goes, under `__snapshots__` beside the test file. Two flags deal with them:

- `--accept` runs the tests with snapshots replaced in place (the `.snap.new` is deleted), so the tests pass. Filters narrow it: `jacket test --accept --exact prints_a_point`.
- `--review` runs nothing. It finds the `.snap.new` files below each root's `tests/` and, in a terminal, shows each as a diff against the stored snapshot (or the whole text when there is none) and asks `[a]ccept, [r]eject, [s]kip, [q]uit`. Without a terminal on standard input it only lists them and exits 1 when there are any. It exits 0 once none are left pending.

Snapshots work the same natively, with `--interp`, and for `-os wasm` (the program reads and writes them itself, so the wasm runner needs access to the folder, which `tools/wasi_run.mjs` has).

### Doc tests

The examples in a library's doc comments are tests too (`crates/jacket/src/test/doctest.rs`). A doc comment is the run of `//` lines (`///` too) or the block comment directly above a constant declaration (a procedure, struct, enum or other `name :: ...`, also inside a top-level `#if`), or the run that opens `module.jai`, which documents the module. jacket asks the compiler for them (`jaic -doc_comments DIR`, which reads each file with jaic's parser): the root's `.jai` files outside `tests/` and outside folders that hold a `module.jai` of their own, then every file they `#load`, wherever it lives, and the `internal/...` modules they `#import` (most standard library modules keep their code in `stdlib/internal`). A block in a loaded file belongs to the root that loads it; a file two roots load is read for the first one only. Only the root's own `module.jai` gives module docs.

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

### Matchers

`assert_that(value, matcher)` checks a value against a description of what it should be (`Extensions/Testing`, `matchers.jai`). Like `expect` it reports at the caller's line, lets the test go on and returns whether it held; `require_that` returns from the test procedure instead. Matchers compose with `not`, `all_of` and `any_of`, and operands may be any type `expect_equal` accepts (numbers of any width compare by value).

```jai
assert_that(count, is(3));
assert_that(name, all_of(starts_with("jac"), has_length(6)));
assert_that(ports, has_item(between(8000, 8999)));
require_that(users, each(not_null()));
```

| Matcher | Holds when |
| --- | --- |
| `is(want)` | equal by value (the `expect_equal` rules); strings, arrays and structs fail with its diff |
| `not(m)` | `m` does not hold |
| `all_of(m...)`, `any_of(m...)` | every / at least one matcher holds |
| `greater_than(v)`, `at_least(v)`, `less_than(v)`, `at_most(v)` | the number (or string, byte by byte) orders that way against `v` |
| `between(low, high)` | `low <= value <= high` |
| `near(want, tolerance)` | `abs(value - want) <= tolerance`; NaN is never near |
| `contains(part)` | a string holds the substring, or an array holds the element |
| `starts_with(x)`, `ends_with(x)` | a string begins / ends with the string, or an array with the elements of the array |
| `is_empty()`, `has_length(n)` | a string or array has no / `n` bytes or elements |
| `has_item(m)`, `each(m)` | some / every array element matches `m` (`each` holds for an empty array) |
| `is_null()`, `not_null()` | a pointer or procedure is / is not null |
| `is_true()`, `is_false()` | a `bool` is that value |
| `of_type(T)` | the value's type is exactly `T` (for an `Any`, the type it holds) |

A matcher that cannot apply to the value (`contains` on an integer) does not hold. A failing check reads like vitest's, naming the failing part of a composite:

```
--- demo (FAIL) ---
  tests/demo.jai:4:
    expected "hello" to contain "hi"
    4 | assert_that("hello", contains("hi"));
  tests/demo.jai:6:
    expected [1, 2, 3] to have every item that should be less than 3
      [2]: expected 3 to be less than 3
    6 | assert_that(s32.[1, 2, 3], each(less_than(3)));
  tests/demo.jai:8:
    expected 5 not to equal 5
    8 | assert_that(5, not(is(5)));
```

`explain(value, matcher)` returns the message instead of reporting it (an empty string when the value matches), which is how the matchers' own tests check message text. Operands are copied into temporary storage, so a matcher belongs to the test that built it. `contains`, `starts_with` and `ends_with` also exist in `String`; both can be imported together, since the matcher versions take one argument and overloading picks the right one.

To add a matcher, add a `Matcher_Kind`, a constructor, and a case each in `holds` and `phrase` (and `failure_message` if it can say more than "expected V to PHRASE") in `matchers.jai`, then a passing and a failing line in `tests/matchers.jai`.

### Filters

`jacket test parse split` runs the tests whose name contains `parse` or `split`; `--exact` makes filters whole names. `name/sub` runs one subtest of `name` (subtests are reported under their test); the cases of a `@cases` test are subtests too. `--partition hash:K/N` keeps the tests whose name hashes (FNV-1a) to part K of N, stable across machines; `count:K/N` takes every Nth by position. `--list` prints `name: kind` per test without running anything (`--format json`: one `{"type":"list",...}` per test). A `@cases` test is followed by a `name/case: subtest` line per case (kind `subtest` in JSON); listing evaluates the lists, which runs their code at compile time.

### Coverage

`jacket test --coverage` reports which lines of the code under test the unit tests ran, and writes them as lcov for editors and CI (`--coverage-out FILE`; default `target/lcov.info` below the project, or `$JACKET_OUTPUT_DIR`).

```
$ jacket test --coverage
...
File                        Lines   Cover
src/Mathy/module.jai          7/8   87.5%
Total                         7/8   87.5%
```

The unit-test program is built with `jaic -coverage` ([coverage](coverage.md)), which makes every test process write the basic blocks it ran at exit. Each process gets its own `JAIC_COVERAGE_DIR` folder that jacket merges and deletes when the process ends. After the run, `llvm-dwarfdump --debug-line` maps blocks to lines (a line is covered when a block holding it ran) and the table is printed to standard output, or to standard error with `--format json`. A coverage error exits with status 2 if the tests passed.

What is reported: files below the test roots and below the folders named by `-import_dir`/`-I` (the project's `import_path`), except the roots' `tests/` folders. A library root's parent folder is not one of them, so the standard library's other modules stay out. Limits: only unit tests are measured (golden, compile-fail and doc programs are not built with `-coverage`); `--interp`, another `-os`/`-cpu` and `jacket bench` are refused; a line is covered or not, with no count; only code compiled into the program shows up, so procedures nothing references are missing from the totals; build unoptimized (the default), since `-O2` merges and moves blocks. `llvm-dwarfdump` comes from `LLVM_SYS_231_PREFIX`, Homebrew's LLVM or `PATH`.

### Watching

`jacket test --watch` runs the tests, then runs them again whenever a `.jai` file changes, until Ctrl-C. Each run clears the screen (on a terminal, for the `human` format) and prints its own summary; a compile error does not end the watch, the next save tries again.

Watched are the test roots and the folders the compiler imports from (`-import_dir`, `-I`, including a library root's parent and the project's `import_path`). Files are polled for a new modification time or size, and a run starts once they have been still for 200 ms, so a save that touches several files runs once. Folders named `target`, `node_modules` and `.git` are skipped, and only `.jai` files count, so the `.snap.new` and temporary files a run writes do not start another. When every changed file is a test file the run is only that file's tests; any other change runs all the selected tests. Filters, `-E`, `--interp` and the rest apply to every run. `--watch` does not combine with `--list` or `--review`, and `jacket bench` has no `--watch`.

### Watching

`jacket test --watch` runs the tests, then runs them again whenever a `.jai` file changes, until Ctrl-C. Each run clears the screen (on a terminal, for the `human` format) and prints its own summary; a compile error does not end the watch, the next save tries again.

Watched are the test roots and the folders the compiler imports from (`-import_dir`, `-I`, including a library root's parent and the project's `import_path`). Files are polled for a new modification time or size, and a run starts once they have been still for 200 ms, so a save that touches several files runs once. Folders named `target`, `node_modules` and `.git` are skipped, and only `.jai` files count, so the `.snap.new` and temporary files a run writes do not start another. When every changed file is a test file the run is only that file's tests; any other change runs all the selected tests. Filters, `-E`, `--interp` and the rest apply to every run. `--watch` does not combine with `--list` or `--review`, and `jacket bench` has no `--watch`.

### Filter expressions

`-E`/`--filter-expr EXPR` selects tests with predicates, as cargo-nextest's filtersets do. It is combined with the positional filters: a test must satisfy both.

```
jacket test -E 'module(Sort) and not kind(property)'
jacket test -E 'file(tests/io/**) | test(/^parse_(int|float)$/)'
jacket test -E 'kind(golden) & !test(slow*)'
```

| Predicate | Selects |
| --- | --- |
| `test(x)` | a test whose name (`Root::name` or `name`) contains `x`; `=x` is the whole name; `x` with `*`, `?`, `[..]` or `{a,b}` is a glob for the whole name; `/re/` is a regular expression found anywhere in it |
| `module(x)` | tests of a root by the module name tests import it as (`Extensions/Testing`) or its folder name; same forms as `test` |
| `file(glob)` | tests in a file, the glob matched against the path from the root, from the current directory and in full |
| `kind(k)` | `unit` (an `@test` that is neither of the next two), `property`, `compile_time`, `snapshot` (the test calls `expect_snapshot`), `golden`, `compile_fail`, `doc`, `bench` |
| `all()`, `none()` | every test, no test |

Operators, tightest first: `not` / `!`, `and` / `&`, `or` / `|`; parentheses group. A mistake is reported with a caret under it and exits with status 2. There is no `tag` predicate since tests carry no tags.

### Seeds

Property tests start from a seed. `--seed N` sets it; otherwise `jacket test` draws a number once per run and gives it to every test program, so all of a run's tests share it (each test mixes in its own name). A failing property prints the seed and `rerun: jacket test --seed N --exact 'name'`, which makes the same inputs again, natively, with `--interp` and for `-os wasm`. A retry (`--retries`) of a failed property uses the same seed and so fails again; a property that is flaky for other reasons is not helped by it.

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
| `@cases(LIST)` | run once per entry of `LIST`, each as a subtest named from its values; see [Testing](../extensions/testing.md#how-it-works) |
| `@property`, `@property(N)` | run once per generated input (100 by default, or `N`); see [Testing](../extensions/testing.md#how-it-works) |
| `@generate(NAME)` | with `@property`: the procedure `NAME` makes the values of the type it returns |
| `@compile_time` | run inside the compiler, once per build, and replayed by the program; see [Testing](../extensions/testing.md#how-it-works) |
| `@retries(N)` | retries for this test, over `--retries` |
| `@only(...)` | run only on these targets: `.LINUX`, `.MACOS`, `.WINDOWS`, `.WASM`, `.ANDROID`, `.X64`, `.ARM64`, `.INTERP`, `.NATIVE`; every kind named must match (`only(.LINUX,.X64)`) |
| `@skip_on(...)` | skip on any of these targets |
| `@needs(.NETWORK)` | skip unless the run has the capability (`--needs network`, `[test] needs`); any name works |
| `@interp_only`, `@native_only` | run only with (without) `--interp` |
| `@exit(N)` | the program must exit with status N during the test, with no failed check |
| `@should_fail`, `@should_fail(word)` | must fail (a check, a crash or a timeout); with a word, a failure message or the test's standard error must contain it |
| `@xfail`, `@xfail(reason)` | a known failure, reported `XFAIL`; passing is an error |
| `@allow_leaks` | the tracking allocator reports no leaks |
| `@args("a", "b $TEMP/x")` | golden tests: the program's command-line arguments (case kept); `$TEMP` is a fresh folder |
| `@warns("text")` | golden tests: the compiler must warn with this text (see above) |
| `@check_only` | golden tests: only type-check the program (`jaic -check`); for code that cannot run on this host |
| `@compile_fail` | file header only: the file below `tests/` is a compile-fail test |

A target may also name an OS and a CPU together (`skip_on(.WINDOWS-ARM64)`), and `.ANDROID` is an OS no host is, so such tests never run here.

An unknown note or a bad argument is a warning naming the file and line; the note is ignored. The target is the host, or `-os`/`-cpu`; a cross-built test is skipped unless `--runner` can run it (compile-fail tests still run).

### Targets

```
jacket test --interp                                   # in jaic's interpreter
jacket test -os wasm --interp                          # in jaic's wasm sandbox
jacket test -os wasm --runner "node --no-warnings tools/wasi_run.mjs" # a wasm build under node
jacket test -- -O2                                     # flags after `--` go to jaic
```

`-os` and `-cpu` are spelled as in jaic (`-os wasm`, `-os=wasm`, `--os wasm`), `-j` and `--jobs` too; an OS or CPU jaic does not know is a usage error. The compiler flags every test build gets are the project's, as for `jacket build`: `[project] import_path` as `-import_dir`, and each `[experimental]` setting the flags after `--` do not already carry. `--runner` is split on spaces, and the built program and its arguments follow its words; it applies to unit and golden tests. Tests run in their own folders, so a word naming an existing file is made absolute: from the current directory for `--runner`, from the folder of `jacket.toml` for `[test] runner`. Flags after `--` go to every jaic run unchanged; the ones that only shape a build's output (`-O2`, `-exe`, ...) are ignored by jaic where jacket checks or interprets instead (listing, compile-fail tests, `--interp`).

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

`kind` is `test`, `golden`, `compile_fail`, `doc` or `bench` (`subtest` in `list` events for the cases of a `@cases` test). `type: "bench"` is reserved for benchmark results. Warnings and build errors stay on standard error.

`--junit FILE` also writes JUnit XML: a `<testsuite>` per root, a `<testcase>` per test with `<failure>`, `<skipped>` and the captured output.

### Editors

`jailsp` and the VS Code extension are built on the JSON above ([VS Code extension](vscode-extension.md#how-it-works), [language server](../compiler/language-server.md)). The language server puts a *Run test* lens over each `@test` procedure, from the parse of the open file, and gives the client the flags that select that test alone: `-E 'test(=NAME) and file(PATH)'` (path from the root, `/` separators), or `--exact NAME` when the path holds a glob or `)` character, after `--path ROOT` for a root `jacket.toml` does not list. The extension lists with `jacket test --list --format json`, builds a file, test, case tree from the `list` events (`subtest` events belong to the test on the same file and line) and runs with `jacket test --format json --exact NAME...`, reading `test` events as they stream. A `failures` entry has a file and line but no column, and the `failed` event of a crash has only `problems` and `stderr`.

### The test program protocol

The program Discovery builds (`stdlib/Extensions/Testing/runner.jai`) takes:

```
tests                       run every test here and print a summary (for running by hand)
tests --list                name the cases of each `@cases` test as `case` records on standard output, run nothing
tests --run NAME... [--temp DIR]   run these tests, the I-th with temp folder DIR/I, and report
tests --seed N              the seed property tests start from (jacket always passes one)
tests --accept              snapshots that differ or are new replace the stored ones
tests --args-from FILE      alone: wait for FILE, then take the arguments from it, one per line (`--interp`, above)
```

With `--run` it writes records to standard error: `\x02`, the kind, each field after `\x1f`, then `\x03` (field bytes `\x02`, `\x03`, `\x1f` are written `?`). The `start` record also goes to standard output, so `jacket` knows which test the output that follows belongs to.

| Record | Fields | Meaning |
| --- | --- | --- |
| `start` | name | a test or subtest (`test/sub`) begins |
| `fail` | name, file, line, message | a check failed |
| `skip` | name, reason | the test skipped itself |
| `end` | name, `pass` / `fail` / `skip` | it finished |
| `missing` | name | no test has this name |
| `case` | test, case | with `--list`, on standard output: one case of a `@cases` test |

Everything else on either stream is the test's own output. A process that stops after `start` without `end` crashed in that test (`exec.rs` turns its exit status or signal into the reason).

### `jacket bench`

Measures the `@bench` procedures; see [jacket bench](jacket-bench.md). `jacket test --list` lists them with kind `bench`; `jacket test` does not run them.

## Migrating from `tests/manifest.json`

Every stdlib module's tests are moving from a `tests/manifest.json` (run by `tools/stdlib/stdlib_runtime.py`) to `jacket test`. A migrated folder is listed under `[test] paths` in `jacket.toml` and loses its manifest; the Python catalogue (`stdlib_test_catalog.py`) skips listed folders, so `stdlib_runtime.py`, `jaic_sweep.py`, `windows_cross.py`, the determinism tool and the browser runner keep working for the rest and stop seeing the migrated ones. `tools/stdlib/migrate_to_jacket_test.py` does the mechanical part.

**Recipe.**

1. `PYTHONPATH=tools python3 tools/stdlib/migrate_to_jacket_test.py --dry-run stdlib/Module`, then without `--dry-run`. It lists what it cannot express and leaves that module alone. Files stay where they are: each case gets a `// test:` header line and expected-output files beside it, and the manifest is deleted.
2. Write the diagnostics of the new compile-fail tests with `jacket test --accept --exact NAME...` (never a bare `--accept`: it also rewrites golden outputs) and read the `.stderr` files.
3. Run `jacket test` for the module in every mode it ran before: native, `--interp`, `--interp -os wasm`, `-os wasm --runner "node --no-warnings tools/wasi_run.mjs"`, and `-- -O2`. Native libraries come from `JAIC_NATIVE_LIBS` as for the old runner.
4. `jaifmt --check`, `jailint -D warnings` on the folder.

Programs run from their own `tests/` folder, as the old runner did. Where a test reads better as `@test` procedures with matchers, rewrite it and delete the program; the move itself does not need that.

| Manifest row | In `jacket test` |
| --- | --- |
| a case (any `tests/*.jai`, or a row with a `source`) | golden test: the file with an expected-output file beside it (`.exit` holding `0` when the row checked nothing else) |
| `stdout`, `stderr`, `exit_code` | `name.stdout`, `name.stderr`, `name.exit`, compared whole |
| `mode: "check-fail"`, `diagnostic_contains` | compile-fail test (`// test: compile_fail`, or a file in `tests/compile_fail/`); the whole diagnostics in `name.stderr` replace the substring |
| `positive_baseline` | dropped: the exact diagnostics show the right error, and the positive case is an ordinary test |
| `mode: "fault"`, `error_contains` | `// test: should_fail("text")`; the exact exit status is not checked (a native abort and the interpreter's status differ) |
| `warning_contains` | `warns("text")` |
| `source_check_only` | `check_only` |
| `required_os`, `required_cpu` | `only(.LINUX,.MACOS)`, `only(.X64)` |
| `unsupported_platforms` (`windows-arm64`) | `skip_on(.WINDOWS-ARM64)` |
| `unsupported_modes` naming both wasm modes, with its `required_capabilities` text | `skip_on(.WASM)` (the wasm legs are `-os wasm --interp` and `-os wasm --runner node`); other combinations: rewrite as `@test`s with `@only`/`@skip_on` notes |
| `arguments`, `input: scratch-file` / `scratch-directory` | `args("literal", "$TEMP/data")` |
| `requires_display` | `needs(.DISPLAY)` (`--needs display`) |
| `window_lease` | `serial` |
| `clipboard_guard` | the test saves and restores the clipboard itself (`defer`) and is `serial`; the Python wrapper is dropped |
| `stdout_one_of` | rewrite as a `@test` that accepts each host's output |
| `input: font` / `ico-fixtures` / `windows-resource-directory`, `fixture_libraries`, `numerics_target_source`, `mode_contracts`, `required_browser_host_capabilities` | not yet: the module stays on the old runner until a feature or fixture step exists |
| the old runner's `-check -no_dce` pass | dropped: every test is built or interpreted |
| `disabled` | fix or delete the case before migrating |

**Other consumers.** `jaic_sweep.py` (differential sweeps), `windows_cross.py` (Windows cross builds), the determinism tool and the browser runner read the catalogue only, so a migrated module leaves their input. Until each reads `jacket test --list --format json` (golden and compile-fail tests carry `file`), migrated modules are covered by the `jacket test` legs of CI (native, `--interp`, `-O2`, and with `WASM=true` both wasm modes under node) and not by those tools: the browser engine and Windows cross builds no longer run them.

**Suites that build a migrated test themselves.** `tools/stdlib/check_foundations.py` and `tools/numerics/run.py` compile some of these programs with their own flags. `tools/stdlib/jacket_tests.py` gives them what the manifest row gave: `expected_output(source)` / `golden_row(source)` read the `.stdout`, `.stderr` and `.exit` beside the program, and `exact_command(source, compiler, env, mode=..., jaic_flags=..., runner=...)` builds the `jacket test --exact` command for a file of `@test` procedures (no `main`, so there is no program to build), in the suite's modes (`--interp`, `-- -O2`, `--os wasm --runner ...`). It starts the `jacket` next to the compiler under test, so build both. `tools/native/windows_cross.py` builds its Windows SDK control once, in the first shard, since the Tls Windows program is no longer one of its cases.

**Leniency.** A fault test no longer checks the exact status or that stdout stayed empty; a program without `main` is type-checked, not run.

## Design decisions

- **Notes are lowercase (`@test`)**, like the language's other notes, and go after the procedure body, where Jai puts them. There is no `test "name" { }` macro: the procedure's name is the test's name, which filters, rerun lines and the editor use. A macro would be a second spelling for the same thing.
- **jacket runs jaic as a program**, not in-process. `jacket` is a front door to separate tools and links none of them; a separate compile also means the tests build exactly as `jaic` builds them. The cost, one compile per root and run, is the same either way.
- **`--interp` isolation is per batch, not per test.** Each `-interp` process compiles the program again, so one per test would cost a compile per test. Tests in a batch still get a fresh tracking allocator, temp folder and temporary storage, and a crash only ends its batch: the next process starts at the next test.
- **Per-test options are notes on the test**; `jacket.toml` has only run-wide settings. A test's requirements move with it, and a test file is readable alone.
- **Doc tests run every ```` ```jai ```` block**, as rustdoc does with Rust's, and need no `@test`: an example that no longer compiles or runs is a failure. Bare statements are wrapped in a procedure, so a short example needs no `main`. Untagged fences are not run, since comments written before doc tests existed use them for other text.
- **Expected-output files hold whole output**, not substrings: a change in a diagnostic shows up as a diff and `--accept` updates it. Paths are written `$DIR` and `$STDLIB`.
- **The design follows** cargo-nextest (process per test, retries and flaky reports, partitions, JSON events), Go's `testing` (`*Test`, subtests, `cleanup`, `temp_dir`), Zig's testing allocator (leak and bad-free checks), pytest (`xfail`), and trybuild (compile-fail files).

## How to change it

- A new flag: `TestArgs` in `crates/jacket/src/cli.rs`, read in `settings` in `crates/jacket/src/test/mod.rs`; if it has a `jacket.toml` key, add it to the `test_settings!` list in `crates/jacket-config/src/schema.rs` (it defines `Test`, `TestRun` and the profile overlay) (and the known-keys lists and the VS Code schema in `editors/vscode/scripts/build.ts`).
- A new note: `TestOptions::apply` and `skip_reason` in `crates/jacket/src/test/options.rs`, then how it is used in `jobs.rs` (`resolve`, `expectations`) or `mod.rs`; list it in the table above.
- Coverage: `coverage.rs` (run files, the line table, the table and lcov), the `-coverage` build in `build::unit`, the per-process folder in `run_unit` (`jobs.rs`) and what is reported in `coverage_for` (`mod.rs`). Counting executions would need a counting runtime in `crates/jaic-llvm/runtime/coverage.c`.
- What is watched, the debounce and which changes narrow a run: `watch.rs`; the run itself is `run_once` in `mod.rs`, which tells `Watched` the folders and test files it used.
- What is watched, the debounce and which changes narrow a run: `watch.rs`; the run itself is `run_once` in `mod.rs`, which tells `Watched` the folders and test files it used.
- A filter predicate: `Parser::predicate` and `eval` in `crates/jacket/src/test/filter.rs`, and what a test offers it in `expr_selects` and `kinds_of` (`mod.rs`).
- A new kind of file test: `Root::read` (`discover.rs`), a `Kind`, and a runner in `golden.rs` called from `run_file_case` (`jobs.rs`).
- Doc tests: what counts as a doc comment and which files are read is `crates/jaic/src/doc_comments.rs` (printed by `jaic -doc_comments`); the fence options and the generated program are in `doctest.rs` (`fenced`, `generate`); checking a block is `doctest::check`.
- Reviewing snapshots: `review.rs` (what `--review` finds, shows and renames). The program side, where a snapshot's file goes and how a value prints, is `snapshots.jai` in the Testing module.
- Scheduling and batching: `jobs` and `run_jobs` in `jobs.rs`. Process handling, record parsing and timeouts: `exec.rs`. Output: `report.rs`.
- The JSON fields are a format others read (CI, the language server, the VS Code extension: `editors/vscode/src/testEvents.ts`): add fields freely, and change or remove one only with a changelog line.
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
runner = "node --no-warnings tools/wasi_run.mjs"   # --runner
needs = ["network"]      # --needs

[test.profile.ci]        # --profile ci: these over [test]
retries = 2
fail_fast = true
```

Flags win over the profile, the profile over `[test]`. Unknown keys get the usual `jacket.toml` warning. Environment: `JAIC_STDLIB` (where jaic finds its standard library, which `$STDLIB` stands for; jacket asks `jaic -print_stdlib`), `TMPDIR` (the work folder).

## Dependencies

`jaic` (found as for every jacket command) and the `Extensions/Testing` module in its standard library; `serde_json` for JSON output and `regex-lite` for `/regex/` filters. `--coverage` needs `llvm-dwarfdump` (LLVM) and `jaic -coverage`. `--runner` programs, such as `node` for `tools/wasi_run.mjs`, are the user's.
