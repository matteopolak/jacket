# `jacket test`

The test runner for Jai projects. It builds a folder's tests once, runs each test in its own process (in parallel), and reports as text, JSON lines or JUnit XML. Tests are written with the [`Testing` module](../extensions/testing.md); golden-output and compile-fail files need no module.

```
$ jacket test
    Starting 12 tests
        PASS [   0.004s] parses_numbers
        FAIL [   0.003s] splits_on_commas
        SKIP             reads_the_registry (only on windows)

--- splits_on_commas (FAIL) ---
  tests/strings.jai:14:
    got:  ["a", "b"]
    want: ["a", "b", "c"]
  rerun: jacket test --exact 'splits_on_commas'

     Summary [   0.120s] 11 tests run: 10 passed, 1 failed, 1 skipped
```

Exit status: 0 when every test passed (or was skipped, or failed under `@xfail`), 1 when a test failed, 2 when nothing matched, the tests didn't compile, or the command line or `jacket.toml` was wrong. Every flag is in the [command-line reference](../reference/command-line.md).

## Where tests are

A test root is a folder with a `tests/` directory (`crates/jacket/src/test/discover.rs`):

| File | Kind |
| --- | --- |
| `tests/name.jai` with `name.stdout`, `name.stderr` or `name.exit` beside it | golden test: built and run alone, output compared |
| `tests/compile_fail/name.jai` (with `name.stderr`) | compile-fail test: must fail to type-check with exactly those diagnostics |
| a `.jai` file in a folder below `tests/` with expected-output files beside it | golden test too (`folder/name`); with `// test: compile_fail` on top, a compile-fail test |
| every other `tests/*.jai` | unit test file: all are `#load`ed into one program whose `@test` procedures are the tests |
| a ```` ```jai ```` block in a doc comment of a library (a root with `module.jai`) | doc test |

Other files below `tests/` are for what the tests `#load` or `#import`.

Two layouts:

- **A library** (`Name/module.jai`, `Name/tests/`): the parent folder is an import folder, so tests `#import "Name"`. Only a library root has doc tests.
- **A project** (`tests/` beside the code, no `module.jai` in the root): `[project] import_path` folders are passed as `-import_dir`, so `import_path = ["src"]` with `src/Name/module.jai` lets tests `#import "Name"`.

Roots are the `--path DIR` folders, else the folders `[test] paths` in `jacket.toml` names (only those the current directory is inside, when it is inside some; of nested roots, the innermost), else the project folder. With several roots, test names get the root's folder as a prefix (`Bit_Array::finds_the_first_set`). `--list` prints the roots that have tests; `--roots` prints every root and runs nothing, for scripts.

An entry of `[test] paths` is a folder or a [path pattern](jailint.md) (`stdlib/**` is every folder below `stdlib`). A pattern selects each folder it matches that has a `tests/` folder or a `module.jai`; nested ones (`stdlib/Android` and `stdlib/Android/Toolchain`) are both roots. The search skips hidden folders, `tests/` folders, and anything `[format] ignore` or `[lint] exclude` match. A root with nothing to run is silent, so a new module needs no entry. A plain folder is taken as written.

Doc examples of `stdlib/internal/*` modules are not run from there: a program cannot import them, so the public module that loads them runs them.

## Writing tests

```jai
#import "Extensions/Testing";

parses_numbers :: () {
    expect_equal(parse_int("42"), 42);
} @test
```

Notes go after the body. A golden or compile-fail file, or every test of a unit file, takes the same options from `// test:` lines at the top of the file:

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
| `@only(...)` | run only on these targets: `.LINUX`, `.MACOS`, `.WINDOWS`, `.WASM`, `.ANDROID`, `.X64`, `.ARM64`, `.INTERP`, `.NATIVE`; every kind named must match |
| `@skip_on(...)` | skip on any of these; `reason = "text"` explains |
| `@needs(.NETWORK)` | skip unless the run has the capability (`--needs network`, `[test] needs`) |
| `@interp_only`, `@native_only` | run only with / without `--interp` |
| `@exit(N)` | the program must exit with status N, with no failed check |
| `@should_fail`, `@should_fail(word)` | must fail; with a word, the failure message or stderr must contain it |
| `@xfail`, `@xfail(reason)` | known failure, reported `XFAIL`; passing is an error |
| `@allow_leaks` | the tracking allocator reports no leaks |
| `@args("a", "b $TEMP/x")` | golden: program arguments. `$TEMP` is a fresh folder, `$DIR` the test file's folder, `${NAME}` an environment variable (must be set, so a missing input is never a silent pass) |
| `@warns("text")` | golden: the compiler must warn with this text |
| `@check_only` | golden: only type-check |
| `@compile_fail` | file header only: the file is a compile-fail test |

`mode = .NATIVE` (or `.INTERP`) and `cpu = .ARM64` (or `.X64`) narrow a call: `skip_on(.WINDOWS, cpu = .ARM64)`, `only(.WASM, mode = .INTERP)`. Without names they mean any OS. A header may hold several calls; every `only(...)` must pass. Under `-os wasm` the CPU is `wasm`, so `only(.X64)` never runs a test as wasm. An unknown note is a warning and is ignored.

### Golden and compile-fail tests

A golden test passes when its exit status equals `name.exit` (default 0), and its stdout and stderr equal `name.stdout` and `name.stderr` when those exist. Line endings are compared as `\n`. `name.stdout.2`, `.3`, ... are other accepted outputs, for machine-dependent results.

A program that must fault at run time is a golden test with `// test: should_fail("text")` on top and a (usually empty) `name.stdout`. A golden program with no `main` is only type-checked, and what the compiler prints is its output.

A compile-fail test is checked with `jaic name.jai -check -quiet -no_color` from its folder and passes when the diagnostics equal `name.stderr`, with the folder written `$DIR` and the standard library `$STDLIB`. `// test: compile_fail("text")` takes no `.stderr` and passes when the diagnostics contain the text.

`--accept` writes the output into the expected files instead of comparing (`.stdout`, the `.stderr`/`.exit` files that exist, or `.stderr` for compile-fail), and replaces snapshots. Narrow it with filters: `jacket test --accept --exact NAME`; a bare `--accept` rewrites every golden output.

### Snapshots

`expect_snapshot` compares a value with a stored snapshot (`__snapshots__/NAME.snap` beside the test file). A differing or missing snapshot fails the test and writes `NAME.snap.new`. `--accept` replaces the snapshots in place. `--review` runs nothing; in a terminal it shows each `.snap.new` as a diff and asks `[a]ccept, [r]eject, [s]kip, [q]uit`, otherwise it lists them and exits 1 if any exist.

### Doc tests

Examples in a library's doc comments are tests (`crates/jacket/src/test/doctest.rs`). The comment is the `//` run or block comment directly above a constant declaration, or the run that opens `module.jai`. jacket gets them from `jaic -doc_comments DIR`. An alias without its own comment forwards to its target, so an example runs once.

```jai
// The pieces of `s` between each `separator`.
//
// ```jai
// parts := split("a,b,c", ",");
// assert(parts.count == 3 && parts[1] == "b");
// ```
split :: Impl.split;
```

Each block is a program importing `Basic` and the library. A block with a file-level `main` is used as is; otherwise its lines become a procedure body with `#import`/`#load` lines moved to file level. A failed `assert`, `expect` or `require` fails the test. Only fences starting with `jai` are tests:

| Fence | The doc test |
| --- | --- |
| ```` ```jai ```` | built and run; must exit 0 |
| ```` ```jai,no_run ```` | only type-checked |
| ```` ```jai,compile_fail ```` | must fail to type-check |
| ```` ```jai,should_fail ```` | built and run; must fail |
| ```` ```jai,ignore ```` | skipped |

The notes above also work as options without the `@`: ```` ```jai,only(linux, macos) ````, `skip_on(wasm)`, `needs(...)`, `timeout(...)`. Doc tests are named `doc: Module.symbol#N`. `--doc` runs only doc tests, `--no-doc` leaves them out.

## Running

```sh
jacket test parse split           # tests whose name contains parse or split
jacket test --exact name          # whole names; name/sub runs one subtest
jacket test -E 'module(Sort) and not kind(property)'
jacket test --list                # names and kinds, runs nothing (--format json for editors)
jacket test --interp              # in jaic's interpreter
jacket test -os wasm --interp     # in jaic's wasm sandbox
jacket test -os wasm --runner "node --no-warnings tools/wasi_run.mjs"
jacket test -- -O2                # flags after -- go to every jaic run
jacket test --watch               # rerun when a .jai file changes
jacket test --coverage            # see below
```

- Each unit test runs as `tests --run NAME` in its own process, `-j` at a time (default: the machine's threads). `@slow` tests start first, `@serial` ones run last, alone. With `--interp` or `--no-isolation`, tests are batched `-j` ways per process, because every `-interp` process compiles again; a crash fails that test and a new process picks up at the next one.
- A test's output is captured and shown with its failure (`--no-capture` streams it, one test at a time). A test past its timeout has its process group killed and is `TIMEOUT`. `--quiet` prints only failures and the summary.
- Each test gets a fresh `temp_dir`; a failing test's is kept and printed (`--keep-temp` keeps all).
- Statuses: `PASS`, `FAIL`, `SKIP`, `TIMEOUT`, `XFAIL`, `XPASS` (fails the run so the note gets removed), `FLAKY` (failed, then passed on a `--retries` attempt; a pass unless `flaky_fails`). `--fail-fast` starts no new tests after the first failure.
- `--partition hash:K/N` keeps tests whose name hashes to part K of N (stable across machines); `count:K/N` takes every Nth.
- `-os`/`-cpu` are spelled as in jaic. A cross-built test is skipped unless `--runner` can run it (compile-fail tests still run). `--runner` is split on spaces; the program and its arguments follow.

### Filter expressions

`-E`/`--filter-expr EXPR` selects with predicates, as cargo-nextest does; it combines with positional filters.

| Predicate | Selects |
| --- | --- |
| `test(x)` | name contains `x`; `=x` whole name; a glob (`*`, `?`, `[..]`, `{a,b}`) for the whole name; `/re/` a regular expression |
| `module(x)` | tests of a root, by the module name or folder name |
| `file(glob)` | tests in a file |
| `kind(k)` | `unit`, `property`, `compile_time`, `snapshot`, `golden`, `compile_fail`, `doc`, `bench` |
| `all()`, `none()` | every / no test |

Operators, tightest first: `not` / `!`, `and` / `&`, `or` / `|`; parentheses group.

### Seeds

Property tests start from a seed. `--seed N` sets it; otherwise one number is drawn per run and shared (each test mixes in its name). A failing property prints `rerun: jacket test --seed N --exact 'name'`.

### Which standard library

Every `jaic` that `jacket test` starts gets `JAIC_STDLIB` (`stdlib_dir` in `crates/jacket/src/test/mod.rs`). The first of these decides: `JAIC_STDLIB` in the environment; the project's own `stdlib/` if it holds the library (so a git worktree tests its own checkout); else what `jaic -print_stdlib` reports. `$STDLIB` in expectations stands for its canonical path.

Native libraries are searched in `<stdlib>/../target/native-libs/<os>-<cpu>`, then in the main checkout's, where `build_native_libs.py` puts them. Missing test inputs that can be built are prepared once before the run (`prepare` in `crates/jacket/src/test/prerequisites.rs`; add a `SETUPS` entry for a new one). A failure list ending in `missing build outputs` names the rest with the command that builds each; `tools/native/ci_native_deps.sh full` builds them all the way CI does.

### Coverage

`jacket test --coverage` reports which lines of the code under test ran and writes lcov (`--coverage-out FILE`, default `target/lcov.info`), or with `--coverage-format html` one self-contained page. `--coverage-by file|module` groups the table; `--coverage-include DIR` (repeatable) adds folders to report (`--coverage-include stdlib` for the whole standard library).

```
File                     Lines   Cover    Functions   Cover
src/Mathy/module.jai       7/8   87.5%          2/2  100.0%
```

Every program the run executes is built with `jaic -coverage` ([coverage](coverage.md)); lines add up across unit tests, golden programs and doc examples. Native coverage needs `llvm-dwarfdump` (`LLVM_PREFIX`, Homebrew LLVM or `PATH`). With `--interp` every program runs as `jaic -interp -coverage` and needs no LLVM. Build unoptimized; `-O2` moves blocks. Procedures nothing references are added as never run.

Baselines keep coverage from dropping: `--coverage-record FILE` writes per-module numbers as JSON, `--coverage-baseline FILE` exits 1 when a module or the total falls more than `--coverage-margin` points (default 0.5) below it, and `--coverage-summary FILE` appends a markdown table. CI runs `tools/ci/stdlib_coverage.sh` against `tools/ci/coverage-baseline.json`; re-record with `tools/ci/stdlib_coverage.sh --record`.

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

## Matchers

`assert_that(value, matcher)` (`Extensions/Testing`, `matchers.jai`) reports at the caller's line, lets the test go on and returns whether it held; `require_that` returns from the test instead.

```jai
assert_that(count, is(3));
assert_that(name, all_of(starts_with("jac"), has_length(6)));
assert_that(ports, has_item(between(8000, 8999)));
require_that(users, each(not_null()));
```

| Matcher | Holds when |
| --- | --- |
| `is(want)` | equal by value (the `expect_equal` rules) |
| `not(m)`, `all_of(m...)`, `any_of(m...)` | `m` doesn't hold / every / at least one holds |
| `greater_than(v)`, `at_least(v)`, `less_than(v)`, `at_most(v)` | orders that way against `v` |
| `between(low, high)` | `low <= value <= high` |
| `near(want, tolerance)` | within tolerance; NaN is never near |
| `contains(part)` | a string holds the substring, or an array the element |
| `starts_with(x)`, `ends_with(x)` | a string or array begins / ends with `x` |
| `is_empty()`, `has_length(n)` | a string or array has no / `n` elements |
| `has_item(m)`, `each(m)` | some / every element matches (`each` holds for an empty array) |
| `is_null()`, `not_null()` | a pointer or procedure is / isn't null |
| `is_true()`, `is_false()` | a `bool` is that value |
| `of_type(T)` | the value's type is exactly `T` |

A matcher that can't apply to the value does not hold. `explain(value, matcher)` returns the failure message (empty when it holds). To add a matcher, add a `Matcher_Kind`, a constructor, and a case each in `holds` and `phrase` (and `failure_message`) in `matchers.jai`, then a passing and a failing line in `tests/matchers.jai`.

## How it works

Per root, `jacket` writes `tests.jai` (one `#load` per unit test file) into a work folder under the temp directory and compiles it:

```
jaic tests.jai -quiet [-os/-cpu/flags] [-import_dir ROOT_PARENT] -exe tests +Extensions/Testing/Discovery
```

The Discovery plugin finds the `@test` and `@bench` procedures, adds the test table and entry point, and prints a `manifest` record per test on stdout. One compile finds the tests and builds them. Roots build in parallel (at most four). A root that doesn't compile stops the run before any test starts.

- **`--interp`** has no program to write. The first `jaic -interp` process runs with the plugin and starts the first tests; jacket then writes `program.jai` (which compiles only what tests reach) for the other processes.
- **`--interp -os wasm`**: the sandbox prints only when the program ends, so the first process runs `-- --list` and every run is a `program.jai` process.
- **`--list`** builds and runs nothing; `--list --deps` prints the source files the compiles read.

The test program (`stdlib/Extensions/Testing/runner.jai`) takes `--list`, `--run NAME... [--temp DIR]`, `--seed N`, `--accept` and `--args-from FILE`. With `--run` it writes records to stderr: `\x02`, the kind, fields separated by `\x1f`, then `\x03`. Kinds: `start`, `fail` (name, file, line, message), `skip`, `end` (`pass`/`fail`/`skip`), `missing`, and `case` (with `--list`, on stdout). Everything else on either stream is the test's own output. A process that stops after `start` without `end` crashed in that test (`exec.rs` turns its status into the reason).

`--watch` polls the test roots and import folders for `.jai` changes and reruns after 200 ms of quiet; if every changed file is a test file only those tests run.

Design choices:

- Notes are lowercase (`@test`) and go after the body, like the language's other notes. The procedure's name is the test's name.
- jacket runs `jaic` as a program, not in-process: it links none of the tools, and tests build exactly as `jaic` builds them.
- Per-test options are notes on the test; `jacket.toml` holds only run-wide settings.
- Doc tests run every ```` ```jai ```` block, so an example that stops compiling is a failure. Untagged fences aren't run.
- Expected-output files hold whole output, not substrings.

## How to change it

- **A flag:** `TestArgs` in `crates/jacket/src/cli.rs`, read in `settings` in `crates/jacket/src/test/mod.rs`; with a `jacket.toml` key, add it to `test_settings!` in `crates/jacket-config/src/schema.rs` and to `editors/vscode/scripts/build.ts`.
- **A note:** `TestOptions::apply` and `skip_reason` in `options.rs`, then `jobs.rs` (`resolve`, `expectations`); list it in the table above.
- **A filter predicate:** `Parser::predicate` and `eval` in `filter.rs`, and `expr_selects`/`kinds_of` in `mod.rs`.
- **A kind of file test:** `Root::read` (`discover.rs`), a `Kind`, and a runner in `golden.rs`.
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
fail_fast = false
flaky_fails = false      # FLAKY fails the run
runner = "node --no-warnings tools/wasi_run.mjs"
needs = ["network"]

[test.profile.ci]        # --profile ci: these over [test]
retries = 2
fail_fast = true
```

Flags win over the profile, the profile over `[test]`. Environment: `JAIC_STDLIB`, `TMPDIR` (the work folder), and `JACKET_TEST_DLLS`, a path list of folders whose `.dll` files are hard-linked next to every native test program (how a Windows test finds `SDL3.dll`, or a replacement `opengl32.dll`).

## Dependencies

`jaic` and the `Extensions/Testing` module; `serde_json` for JSON output and `regex-lite` for `/regex/` filters. `--coverage` needs `llvm-dwarfdump` and `jaic -coverage`. `--runner` programs such as `node` are the user's.
