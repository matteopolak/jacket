# `jacket test`

The test runner for Jai projects. It builds a folder's unit programs once, runs their cases and separate golden programs in parallel, and reports as text, JSON lines or JUnit XML. Tests are written with the [`Testing` module](../extensions/testing.md); golden-output and compile-fail files need no module. How to use it is in the [testing guide](../guides/testing.md); this page is the reference and how it works.

Exit status: 0 when every test passed (or was skipped, or failed under `@xfail`), 1 when a test failed, 2 when nothing matched, the tests didn't compile, or the command line or `jacket.toml` was wrong. Every flag is in the [command-line reference](../reference/tools.md#testing).

`--list` shows the host's catalogue. `--list --list-target` makes it match the selected target's compile-time `#if OS` and `#if CPU`; pass the same compiler flags as the run (`jacket test --list --list-target --interp -- -os=wasm`). The listing view is chosen in `test/build.rs`, apart from the shared compiler argument and target policy. CI uses the selected-target view for coverage and changed-file filters.

## Where tests are

A test root is a folder with a `tests/` directory (`crates/jacket/src/test/discover.rs`):

| File | Kind |
| --- | --- |
| `tests/name.jai` with `name.stdout`, `name.stderr` or `name.exit` beside it | golden test: built and run alone, output compared |
| `tests/compile_fail/name.jai` (with `name.stderr`) | compile-fail test: must fail to type-check with exactly those diagnostics |
| a `.jai` file in a folder below `tests/` with expected-output files beside it | golden test too (`folder/name`); with `// test: compile_fail` on top, a compile-fail test |
| every other `tests/*.jai` | unit test file: all are `#load`ed into one program whose `@test` procedures are the tests |
| a ```` ```jai ```` block in a doc comment of a library (a root with `module.jai`) | doc test |

Other files below `tests/` are for what the tests `#load` or `#import`. All unit test files are one program, so two of them cannot `#load` the same helper file (`module-already-loaded`); put shared helpers in a folder with a `module.jai` and `#import,dir` it from each.

Two layouts:

- **A library** (`Name/module.jai`, `Name/tests/`): the parent folder is an import folder, so tests `#import "Name"`. Only a library root has doc tests.
- **A project** (`tests/` beside the code, no `module.jai` in the root): `[project] import_path` folders are passed as `-import_dir`, so `import-path = ["src"]` with `src/Name/module.jai` lets tests `#import "Name"`.

Roots are the `--path DIR` folders, else the folders `[test] paths` in `jacket.toml` names (only those the current directory is inside, when it is inside some; of nested roots, the innermost), else the nearest folder from the current directory up to (not including) the project folder that has a `tests/` folder or a `module.jai`, else the project folder. So `jacket test` in `examples/kvdb` tests `kvdb` when only `examples/` has a `jacket.toml`. With several roots, test names get the root's folder as a prefix (`Bit_Array::finds_the_first_set`). `--list` prints the roots that have tests; `--roots` prints every root and runs nothing, for scripts.

An entry of `[test] paths` is a folder or a [path pattern](jailint.md) (`stdlib/**` is every folder below `stdlib`). A pattern selects each folder it matches that has a `tests/` folder or a `module.jai`; nested ones (`stdlib/Android` and `stdlib/Android/Toolchain`) are both roots. The search skips hidden folders, `tests/` folders, and anything `[format] ignore` or `[lint] exclude` match. A root with nothing to run is silent, so a new module needs no entry. A plain folder is taken as written.

Doc examples of `stdlib/internal/*` modules are not run from there: a program cannot import them, so the public module that reaches them runs them, through `#load` or through `#import` of an `internal/...` module at any depth. A file two public modules reach runs under the first root in path order.

## Writing tests

The [testing guide](../guides/testing.md) has examples of tests, checks, `@cases` and `@property`. Notes go after the body. A golden or compile-fail file, or every test of a unit file, takes the same options from `// test:` lines at the top of the file:

```jai
// test: only(.LINUX,.MACOS) timeout(10)
```

| Note | Effect |
| --- | --- |
| `@timeout(N)` | kill after N seconds (0: no limit); default `--timeout`, else `[test] timeout`, else 120 |
| `@slow` | known slow: started first, no `SLOW` line |
| `@serial` | runs while no other test in this run does |
| `@cases(LIST)` | once per entry, each a subtest ([Testing](../extensions/testing.md#how-it-works)) |
| `@property`, `@property(N)` | once per generated input (100 by default); `@generate(NAME)` supplies values |
| `@compile_time` | run inside the compiler, once per build, and replayed by the program |
| `@retries(N)` | retries for this test |
| `@only(...)` | run only on these targets: `.LINUX`, `.MACOS`, `.WINDOWS`, `.WASM`, `.ANDROID`, `.X64`, `.ARM64`, `.INTERP`, `.NATIVE`, `.CHECK`; every kind named must match |
| `@skip_on(...)` | skip on any of these; `reason = "text"` explains |
| `@needs(.NETWORK)` | skip unless the run has the capability (`--needs network`, `[test] needs`) |
| `optional(NAME)` | skip, instead of failing, when an `args(...)` entry uses `${NAME}` and the variable is unset (`optional(JAIC_TEST_FONT_PATH)`) |
| `@interp_only`, `@native_only` | run only with / without `--interp`; a `--check-only` run counts as without |
| `@exit(N)` | the program must exit with status N, with no failed check |
| `@should_fail`, `@should_fail(word)`, `@should_fail("a", "b")` | must fail; with text, the failure message or stderr must contain it (any one of several when the note starts with a quote) |
| `@xfail`, `@xfail(reason)` | known failure, reported `XFAIL`; passing is an error |
| `@allow_leaks` | the tracking allocator reports no leaks |
| `@args("a", "b $TEMP/x")` | golden: program arguments. `$TEMP` is a fresh folder, `$DIR` the test file's folder, `${NAME}` an environment variable (must be set, so a missing input is never a silent pass) |
| `@warns("text")` | golden: the compiler must warn with this text |
| `@check_only` | golden: only type-check |
| `@compile_fail` | file header only: the file is a compile-fail test |

`mode = .NATIVE` (`.INTERP`, or `.CHECK` for a `--check-only` run) and `cpu = .ARM64` (or `.X64`) narrow a call: `skip_on(.WINDOWS, cpu = .ARM64)`, `only(.WASM, mode = .INTERP)`. Without names they mean any OS. A header may hold several calls; every `only(...)` must pass. Under `-os wasm` the CPU is `wasm`, so `only(.X64)` never runs a test as wasm. An unknown note is a warning and is ignored. Spaces inside a note's parentheses work (`@skip_on(.MACOS, reason = "needs a display")`) but jaic warns that the official compiler rejects them (`note-with-spaces`); a `// test:` line takes them silently.

### Golden and compile-fail tests

A golden test passes when its exit status equals `name.exit` (default 0), and its stdout and stderr equal `name.stdout` and `name.stderr` when those exist. Line endings are compared as `\n`. `name.stdout.2`, `.3`, ... are other accepted outputs, for machine-dependent results.

A program that must fault at run time is a golden test with `// test: should_fail("text")` on top and a (usually empty) `name.stdout`. A golden program with no `main` is only type-checked, and what the compiler prints is its output.

A compile-fail test is checked with `jaic name.jai -check -quiet -no_color` from its folder and passes when the diagnostics equal `name.stderr`, with the folder written `$DIR`, the standard library `$STDLIB` and the project folder (the one with `jacket.toml`) `$ROOT`, and each absolute `-import_dir` folder `$IMPORT1`, `$IMPORT2`, ... in flag order. Only whole paths are replaced (`/r` does not match inside `/rs`), the first of `$STDLIB`, `$DIR`, `$ROOT`, `$IMPORTn` that fits wins, and separators after a placeholder become `/`. A path outside all of them, such as a system library, stays as printed, so a test should not depend on one. `// test: compile_fail("text")` takes no `.stderr` and passes when the diagnostics contain the text.

A compile-fail test with no `.stderr` fails. `jacket test --create-missing` writes the missing files from what the compiler printed and says `CREATED <file>`; check them and commit them. (`--accept` writes them too, and rewrites the ones that differ.) (A `.jai` file below `tests/` with no expected file at all is not a test.)

`--accept` writes the output into the expected files instead of comparing (`.stdout`, the `.stderr`/`.exit` files that exist, or `.stderr` for compile-fail), and replaces snapshots. Narrow it with filters: `jacket test --accept --exact NAME`; a bare `--accept` rewrites every golden output.

### Snapshots

`expect_snapshot` stores `__snapshots__/NAME.snap` beside the test file; a differing or missing snapshot fails the test and writes `NAME.snap.new`. `--accept` replaces snapshots in place and `--review` (`review.rs`) goes through the `.snap.new` files. Usage is in the [guide](../guides/testing.md#snapshots).

### Doc tests

Examples in a library's doc comments are tests (`crates/jacket/src/test/doctest.rs`). The comment is the `//` run or block comment directly above a constant declaration, or the run that opens `module.jai`. jacket gets them from `jaic -doc_comments DIR`, passing the `-import_dir` folders the tests compile with, so an alias of a module in one of them takes that module's docs; if `jaic` can't be started or can't read a library, the run fails instead of testing no examples. An alias without its own comment forwards to its target, so an example runs once.

- Each block is a program importing `Basic` and the library. A block with a file-level `main` is used as is; otherwise its lines become a procedure body with `#import`/`#load` lines moved to file level.
- A failed `assert`, `expect` or `require` fails the test and shows the example's line and code. The Testing module prints each as `FILE:LINE: MESSAGE` (later lines indented four spaces) and `doctest::reported_failures` reads them from the example's standard error. A failed `assert` ends its test with that report instead of the stack trace, and the tests after it run in a new program.
- Only fences starting with `jai` are tests. The fence options (`no_run`, `compile_fail`, `should_fail`, `ignore`) are in the [guide](../guides/testing.md#doc-tests); the notes above also work as options without the `@`: ```` ```jai,only(linux, macos) ````, `skip_on(wasm)`, `needs(...)`, `timeout(...)`.
- A `// test:` line in its own comment run at the top of `module.jai` (after the module docs, with a blank line on each side) applies to every doc test of the library, so a module with no wasm target writes `// test: skip_on(.WASM, reason = "needs threads")` once. It is a header like a test file's and is not part of the module docs.
- Doc tests are named `doc: Module.symbol#N`. `--doc` runs only doc tests, `--no-doc` leaves them out.

### Checking examples for another OS

`--check-only --os <os>` type-checks every unit test, golden test and example for that target from any machine, without building, linking or running (`jacket test --check-only --os windows`; `--cpu` picks the CPU). It catches mistakes that only exist on another OS, such as a struct member Windows names differently.

- `only(...)` and `skip_on(...)` notes on a fence, and the `// test:` header of `module.jai`, are matched against `--os`, not the host.
- Every example is checked as `no_run` is: a `compile_fail` example must still not compile and a `should_fail` example must compile. Mark a limit of the target with a note (in the module header when the whole module lacks the target); otherwise fix the example.
- A unit test root is checked as a whole with every procedure kept (`-no_dce`). `--doc` limits the check to examples.
- A golden test's `#run` code still runs, so a check that fails is held to its `should_fail` or `xfail`. A test whose `#run` builds and runs a program says `skip_on(mode = .CHECK, reason = "...")`, since `-check` writes no output for the workspaces a metaprogram creates.

CI runs it for the four OSes in the stdlib type-check job ([testing](testing.md)).

## Running

Examples of every flag are in the [guide](../guides/testing.md#running-tests).

- Each unit test runs as `tests --run NAME` in its own process, `-j` at a time (default: the machine's threads). `@slow` tests start first, `@serial` ones run last, alone. With `--interp` or `--no-isolation`, tests are batched `-j` ways per process, because every `-interp` process compiles again; a crash fails that test and a new process picks up at the next one.
- A test's output is captured and shown with its failure (`--no-capture` streams it, one test at a time). A test past its timeout has its process group killed and is `TIMEOUT`. Plain golden programs use their declared timeout from process launch, even without Testing records; framed unit programs retain a separate deadline for each test. Closing both output streams does not end supervision: Jacket waits for the process and checks its completion before declaring an expired deadline. A timeout of 0 disables that limit. `--quiet` prints only failures and the summary.
- A test whose process dies of a signal (or a Windows exception such as `0xC0000005`) shows the last 64 lines of its stdout and stderr under the failure, for unit tests and golden tests alike. Native test builds pass `-backtrace_on_crash`, so stderr also has `crash signal N at 0xADDR` and a `native stack:` trace (`stdlib/internal/runtime/crash-native/runtime.c`). `-no_backtrace_on_crash` after `--` turns the trace off; `--interp` runs have none.
- Each test gets a fresh `temp_dir`; a failing test's is kept and printed (`--keep-temp` keeps all).
- Statuses: `PASS`, `FAIL`, `SKIP`, `TIMEOUT`, `XFAIL`, `XPASS` (fails the run so the note gets removed), `FLAKY` (failed, then passed on a `--retries` attempt; a pass unless `flaky-fails`). `--fail-fast` starts no new tests after the first failure.
- `--changed FILE` (repeat it per file) keeps the tests a change to FILE can affect, before `--partition` splits what is left. A unit-test root, a golden program and a compile-fail test are kept when their program reads FILE (the `--list --deps` closure); a compile-fail test and a root's doc examples cannot be compiled alone, so they stand for the modules they `#import`. A test whose program does not compile is kept (a deleted file breaks its importers), and so is every test of a root that gains a file no program reads (a new test, a data file). `Basic` changes keep nearly everything. What a program reads comes from the [dependency store](#dependency-store), so a warm store costs no compiles. Pass paths as they exist from the current directory. CI's use of it: [CI job selection](ci-job-selection.md).
- `-os`/`-cpu` are spelled as in jaic. A cross-built test is skipped unless `--runner` can run it (compile-fail tests still run). `--target TRIPLE` for another machine, with no runner, fails and names the fix. `--runner` is split on spaces; the program and its arguments follow.

### Partitions

`--partition hash:K/N` keeps tests whose name hashes to part K of N (stable across machines); `count:K/N` takes every Nth. `time:K/N` with `--partition-times FILE` (a JSON object of seconds per test name) deals the tests, slowest first, to the part with the least time so far, so the parts take about equal time; a test missing from the file weighs the median of those listed. The split depends only on the set of names and the times, so each part computes it on its own. CI's table is `tools/ci/test-times.json` (built by `tools/ci/test_times.py` from `--junit` files).

### Filter expressions

`-E`/`--filter-expr EXPR` selects with predicates and combines with positional filters.

| Predicate | Selects |
| --- | --- |
| `test(x)` | name contains `x`; `=x` whole name; a glob (`*`, `?`, `[..]`, `{a,b}`) for the whole name; `/re/` a regular expression |
| `module(x)` | tests of a root, by the module name or folder name |
| `file(glob)` | tests in a file |
| `kind(k)` | `unit`, `property`, `compile_time`, `snapshot`, `golden`, `compile_fail`, `doc`, `bench` |
| `all()`, `none()` | every / no test |

Operators, tightest first: `not` / `!`, `and` / `&`, `or` / `|`; parentheses group.

`--seed N` sets the property-test seed; otherwise one number is drawn per run and each test mixes in its name.

### Which standard library

Every `jaic` that `jacket test` starts gets `JAIC_STDLIB` (`stdlib_dir` in `crates/jacket/src/test/mod.rs`). The first of these decides: `JAIC_STDLIB` in the environment; the project's own `stdlib/` if it holds the library (so a git worktree tests its own checkout); else what `jaic -print_stdlib` reports. `$STDLIB` in expectations is its canonical path and `$ROOT` the project folder.

Native libraries are searched in `<stdlib>/../target/native-libs/<os>-<cpu>`, then in the main checkout's, where `build_native_libs.py` puts them.

**Prepared inputs.** Missing inputs that can be built are prepared once before the run: the fixture data, the machine fixture, the conformance data the Time, Csv, Unicode_Text, Archive and Gltf (validator) tests read, and the Debug helper. All but the helper come from one `build_portable.py data` run, which downloads pinned archives. A test needs a setup when its code (comments ignored) names the setup's fixture directory. They are looked for in `JACKET_OUTPUT_DIR` when set, else in `target/`. The Gltf sample models are not part of `data`; fetch them with `tools/gltf_conformance/fetch.py --samples`. If `python3` is missing or a build script fails, the run stops with the command to run by hand (`prepare` in `crates/jacket/src/test/prerequisites.rs`; add a `SETUPS` entry for a new setup). A failure list ending in `missing build outputs` names the rest with the command that builds each; `tools/native/ci_native_deps.sh full` builds them all the way CI does.

On macOS and Linux the run also builds the small C probe libraries some stdlib test drivers link (`thread_sdk_identity`, and on macOS `unmapping_protection` and `window_gl_probe`) with `cc` and `ar`, into `jacket-test-probes/` of the output directory. A probe that does not build stops the run with the failing command. `--allow-missing-prerequisites` turns a missing probe or prepared input into a skip, each with the command that builds it.

### Coverage

`jacket test --coverage` reports which lines of the code under test ran. It writes lcov (`--coverage-out FILE`, default `target/lcov.info`, or in `JACKET_OUTPUT_DIR`; a relative `JACKET_OUTPUT_DIR` is below the checkout the standard library is in) or, with `--coverage-format html`, one self-contained page. `--coverage-by file|module` groups the table and `--coverage-include DIR` adds folders to report (`stdlib` for the whole standard library). Files under `tests` or `benches` are not reported.

Every program the run executes is built with `jaic -coverage` ([coverage](coverage.md)), and lines add up across unit tests, golden programs and doc examples. Native coverage needs `llvm-dwarfdump` (`LLVM_PREFIX`, Homebrew LLVM or `PATH`); with `--interp` every program runs as `jaic -interp -coverage` and needs no LLVM. Build unoptimized, since `-O2` moves blocks. Procedures nothing references are added as never run.

A test process writes its coverage as it exits on its own. One that ends without (`_exit`, a handler that never returns) or whose file is cut short makes the run fail with exit code 2 after the report is written, naming the processes, and no baseline is compared or recorded. Look at how that program exits before suspecting the report.

**Baselines** keep coverage from dropping. `--coverage-record FILE` writes per-module numbers as JSON; `--coverage-baseline FILE` exits 1 when a module or the total falls more than `--coverage-margin` points (default 0.5) below it and runs more than 5 lines fewer than the baseline's share would; `--coverage-summary FILE` appends a markdown table. CI runs `tools/ci/stdlib_coverage.sh` against `tools/ci/coverage-baseline.json` with `--seed 1`; re-record with `tools/ci/stdlib_coverage.sh --record`. The `coverage-baseline` artifact holds the measured JSON and `stdlib-coverage.lcov`; diff the lcov of two runs to see which lines moved. On Linux and other non-Apple targets the DWARF 5 line table keeps every file's lines only because the compile unit's file has a name of its own ([debug info](../native/debug-info.md)); if a module's whole `module.jai` reads 0%, inspect `llvm-dwarfdump --debug-line` for its folder.

## Tests on a board

`jacket test --target Qemu_Virt_Aarch64` (or a `[target.NAME]` with a `board`, [targets](jacket.md#targets)) builds the tests for the board and runs each group in an emulator. The result comes back as records on the board's semihosting console, whose standard output and standard error stay apart, and the arguments go in through the semihosting command line (`-append {args}`). Only unit tests run: golden tests and doc examples that run build host programs, so they are reported as skipped. Compile-fail tests and doc examples that only compile need no runner and run, checked for the host without the board's plugin. Everything in the root has to compile for `OS == .NONE`: put such tests in a folder of their own and select it with `--path`. Benchmarks need a clock and do not run, snapshots and temporary folders need files and are unavailable, and `--args-from-stdin` does not apply.

```sh
jacket test --target Qemu_Virt_Aarch64 --path stdlib/Extensions/Embedded
```

## Output formats

`--format json` writes one JSON object per line as things happen:

| `type`, `event` | Fields |
| --- | --- |
| `run`, `started` | `tests` |
| `test`, `started` | `name`, `kind` |
| `test`, `slow` | `name`, `seconds` |
| `test`, `passed` / `failed` / `skipped` / `timed_out` / `xfail` / `xpass` / `flaky` | `name`, `kind`, `file`, `line`, `duration`, `attempts`, `failures` (`test`, `file`, `line`, `message`), `problems`, `subtests` (`name`, `status`, `duration`), `stdout`, `stderr`, `reason`, `temp_dir` |
| `run`, `finished` | `passed`, `failed`, `skipped`, `timed_out`, `xfail`, `xpass`, `flaky`, `not_run`, `duration` |
| `list` | `name`, `kind`, `file`, `line`, `notes` (with `--list`) |

`kind` is `test`, `golden`, `compile_fail`, `doc` or `bench` (`subtest` for `@cases` cases in `list`). `--junit FILE` also writes JUnit XML. The VS Code extension and `jailsp` are built on this JSON ([VS Code extension](vscode-extension.md)); the fields are a format others read, so add freely and change or remove only with a changelog line.

## How it works

Per root, `jacket` writes `tests.jai` (one `#load` per unit test file) into a work folder under the temp directory and compiles it:

```
jaic tests.jai -quiet [-os/-cpu/flags] [-import_dir ROOT_PARENT] -exe tests +Extensions/Testing/Discovery
```

The Discovery plugin finds the `@test` and `@bench` procedures, adds the test table and entry point, and prints a `manifest` record per test on stdout. One compile finds the tests and builds them. Roots build in parallel (at most four). A root that doesn't compile stops the run before any test starts.

- **`--interp`** has no program to write. The first `jaic -interp` process runs with the plugin and starts the first tests; jacket then writes `program.jai` (which compiles only what tests reach) for the other processes.
- **`--interp` roots beyond `-j`**: only the first `-j` roots keep their compiling process waiting (`warm` in `build::unit`); each waiting process is a whole compiler in memory. The others compile again when they run, as `program.jai` processes do.
- **`--interp -os wasm`**: the sandbox prints only when the program ends, so the first process runs `-- --list` and every run is a `program.jai` process.
- **`--list`** builds and runs nothing; `--list --deps` prints the source files the compiles read.
- **Dependency lists:** every compile above, and each golden program's build or `--interp` run, also passes `-deps_file` (see [Dependency store](#dependency-store)).

The test program (`stdlib/Extensions/Testing/runner.jai`) takes `--list`, `--run NAME... [--temp DIR]`, `--seed N`, `--accept` and `--args-from-stdin` (block until standard input ends, then read the arguments from it; how `--interp` hands its first tests to the process that compiled them). With `--run` it writes records to stderr: `\x02`, the kind, fields separated by `\x1f`, then `\x03`. Kinds: `start`, `fail` (name, file, line, message), `skip`, `end` (`pass`/`fail`/`skip`), `missing`, and `case` (with `--list`, on stdout). Everything else on either stream is the test's own output. A process that stops after `start` without `end` crashed in that test (`exec.rs` turns its status into the reason).

`--watch` polls the test roots and import folders for `.jai` changes and reruns after 200 ms of quiet; if every changed file is a test file only those tests run.

### Dependency store

`--changed` needs the source files each test program reads. The compiles jacket does anyway (a root's unit-test build or listing, a golden program's build, check or `--interp` run) pass `jaic -deps_file <entry>`. The store is the folder of those entries: `jacket/test-deps` in the user cache folder, or `JACKET_TEST_DEPS_DIR` (CI points it at a cached folder). With neither a cache folder nor the variable, or a store it cannot read or write, `jacket test` stops with an error. A compiler that does not start while `--changed` lists a program's reads is an error, not a reason to keep every test.

An entry is named by a hash of the `jaic` binary's contents, the standard library path, the program (its path, or its text for a generated program such as a root's `tests.jai`), the compiler flags, the root's flags, and whether the Discovery plugin runs. The mode (build, `-interp`, `-check`) and the `-add` that points at the prepared probe libraries are not part of it.

`--changed` reads the store first. A program with no entry is compiled with `jaic -check -deps_file`, `-j` programs at a time; this covers compile-fail tests and doc examples through a stand-in program of their imports. A compile that fails leaves no entry, and a program with no list is always kept.

A stored list can be older than the files, which is sound for selection: a program starts reading a new file only if a file it already read changed to import or load it, and that file is in the old list. A new file that shadows module resolution is the unlisted file of a root, which `--changed` selects. This holds within one checkout; cache one store per checkout history, not across unrelated branches. Entries not used for 30 days are deleted when the store is opened; remove the folder to start over.

## How to change it

- **A flag:** `TestArgs` in `crates/jacket/src/cli.rs`, read in `settings` in `crates/jacket/src/test/mod.rs`; with a `jacket.toml` key, add it to `test_settings!` in `crates/jacket-config/src/schema.rs` and to `editors/vscode/scripts/build.ts`.
- **A note:** `TestOptions::apply` and `skip_reason` in `options.rs`, then `jobs.rs` (`resolve`, `expectations`); list it in the table above.
- **A filter predicate:** `Parser::predicate` and `eval` in `filter.rs`, and `expr_selects`/`kinds_of` in `mod.rs`.
- **A kind of file test:** `Root::read` (`discover.rs`), a `Kind`, and a runner in `golden.rs`.
- **A matcher:** add a `Matcher_Kind`, a constructor, and a case each in `holds`, `phrase` and `failure_message` in `stdlib/Extensions/Testing/matchers.jai`, then a passing and a failing line in `tests/matchers.jai`; list it in the [guide](../guides/testing.md#matchers).
- **Doc tests:** `crates/jaic-syntax/src/doc_comments.rs` decides what counts; fence options and the generated program are in `doctest.rs`.
- **Coverage:** `coverage/` (`mod.rs`, `dwarf.rs`, `procedures.rs`, `report.rs`, `baseline.rs`, `html.rs`).
- **Scheduling, processes, output:** `jobs.rs`, `exec.rs`, `report.rs`; watching is `watch.rs`; snapshot review is `review.rs`.
- Run the runner's unit tests with `cargo test -p jacket`, and the Testing module's with `jacket test` in `stdlib/Extensions/Testing`.

## Configuration

```toml
[test]
paths = ["stdlib/**"]    # test roots, folders or patterns; default: the project folder
jobs = 8                 # -j
timeout = 120            # seconds; 0: no limit
slow = 30                # seconds before a SLOW line
retries = 0
fail-fast = false
flaky-fails = false      # FLAKY fails the run
runner = "node --no-warnings tools/wasi_run.ts"
needs = ["network"]

[test.profile.ci]        # --profile ci: these over [test]
retries = 2
fail-fast = true
```

Flags win over the profile, the profile over `[test]`. Environment: `JAIC_STDLIB`, `TMPDIR` (the work folder), and `JACKET_TEST_DEPS_DIR` (the [dependency store](#dependency-store)), and `JACKET_TEST_DLLS`, a path list of folders whose `.dll` files are hard-linked next to every native test program (how a Windows test finds `SDL3.dll`, or a replacement `opengl32.dll`).

## Dependencies

`jaic` and the `Extensions/Testing` module; `serde_json` for JSON output and `regex-lite` for `/regex/` filters. `--coverage` needs `llvm-dwarfdump` and `jaic -coverage`. `--runner` programs such as `node` are the user's.

## Effective compiler target

Target filters, cross-target skips and runner routing use the final compiler arguments: `jacket test --os windows -- -os=wasm` selects wasm tests and needs a wasm runner. Repeated OS or CPU flags keep the last value, and `-target` overrides both whatever the order. CPU aliases and defaults come from the compiler's shared `Selection` policy; cross-OS checking defaults to x64 and wasm builds have the wasm CPU in every mode.

- Host-side discovery removes target options, including inline values and triples, using the option table of `jaic-cli-args`. Values of unrelated flags, plugin options and arguments after `-` or `--` keep their meaning. Target-side builds keep the user's argument bytes.
- A metaprogram that changes its workspace target is outside this selection; its target can't be known before it runs.
- Forwarded compiler mode switches are resolved once, then removed before the per-case commands are built. Compile-fail cases keep `-check` during an interpreted run, and host discovery keeps its own interpreter mode. Conflicts such as `-check -interp` stay errors, including with `--check-only`. Forwarded `-interp` selects interpreted execution too. Use `jacket test --list --deps` for dependency lists: forwarding `-deps` is rejected.
- Compile-only cases use checking target rules, and native link support is validated when a program is built, so compile-fail and check-only examples can select cross-target filters without a foreign linker.

To extend target selection, update the shared policy in `jaic::target` and the option metadata in `jaic-cli-args`, then exercise `crates/jacket/tests/forwarded_targets.rs`; keep selection and build/runner behaviour in the same regression.
