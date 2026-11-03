# Tools

`jacket` is the one command for the toolchain: it fronts the compiler (`jaic`), the linter (`jailint`), the formatter (`jaifmt`) and the language server (`jailsp`), and adds projects, tests, benchmarks and documentation. Each page lists a command's flags from its own `--help`.

```text
jacket <COMMAND>
```

A command `jacket` does not know runs `jacket-<command>` from `PATH`, as `cargo` does.

## Building

### Compiling

`jaic` compiles, checks and runs Jai programs. `jacket build`, `check` and `run` pass the flags of `jaic` through untouched (single-dash, as in `jacket build -o -exe app main.jai`) and add the project: with no `.jai` file named they build `[project] entry` from the nearest `jacket.toml` (else `build.jai`, `first.jai`, `main.jai`, `src/main.jai` or `module.jai`), and `[experimental]` settings become `jaic` flags unless you pass them. [The `jaic` command line](../tools/cli.md) says how the flags fit together.

#### Commands

| Command | Meaning |
| --- | --- |
| `jacket build` | Compile a program: `jaic <args>`, flags untouched |
| `jacket check` | Type-check without writing anything: `jaic -check <args>` |
| `jacket run` | Build, then run the program; arguments after `--` go to it (`-interp` runs it in jaic) |
| `jacket clean` | Remove the executable `jacket build` writes |
| `jacket jaic` | Run `jaic` with exactly these arguments |

#### jaic

jaic: compile, check and run Jai programs

```text
jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]
```

By default jaic builds a native executable. Options may come before or after the file.

##### Modes

| Option | Meaning |
| --- | --- |
| `-check` | compile the program without running or writing anything |
| `-interp` | run `main` in the interpreter (arguments after `--` go to the program) |
| `-deps` | like -check, then print every source file the compile read, one per line |

##### Program

| Option | Meaning |
| --- | --- |
| `-add "code"` | add the code to the target program (repeatable) |
| `-run "code"` | run the code at compile time, as a `#run` (repeatable) |
| `-import_dir <dir>` | also look for modules in &lt;dir&gt; (repeatable; short form -I) |
| `-os <os>` | target OS: linux, windows, macos or wasm |
| `-cpu <cpu>` | target CPU: x64 or arm64 |
| `-target <triple>` | build for an LLVM target triple (overrides -os and -cpu) |
| `-no_dce` | type-check unreferenced module code too |
| `-no_check` | accepted and ignored: jaic does not run the Check plugin unless asked |
| `-no_check_bindings` | set CHECK_BINDINGS=false on a +Check plugin, if there is one |

##### Output (building only)

| Option | Meaning |
| --- | --- |
| `-j, --jobs <n>` | run n compiles (or tests) at once (default: one per CPU) |
| `-exe <name>` | the executable's name; it may include a directory and extension (`-exe build/app.exe`). A bare name goes in the current directory |
| `-output_path <dir>` | the directory the executable is written to |
| `-o, -optimized` | optimized build (LLVM O2) without stack traces or runtime checks |
| `-od, -optimized_debug` | lightly optimized build (LLVM O1) that keeps stack traces and checks |
| `-very_debug` | unoptimized build with every runtime check on |
| `-O0, -O1, -O2, -O3` | pick the LLVM optimization level directly |
| `-no_inline` | do not inline procedures |
| `-no_split` | generate code as one module instead of several |
| `-backtrace_on_crash` | install the crash handler (a metaprogram's setting wins; -no_backtrace_on_crash overrides) |
| `-no_backtrace_on_crash` | leave the crash handler out of the program |
| `-emit_ir <file.ll>` | also write the LLVM IR |
| `-no_debug_info` | leave out native debug information |
| `-sanitize <list>` | address, undefined and/or fuzzer, comma-separated |
| `-coverage` | record line and function coverage (docs/tools/coverage.md) |
| `-no_workspace_output` | (-interp only) do not write what a metaprogram's workspaces ask for |
| `-codegen_units <n>` | generate code as exactly &lt;n&gt; modules (1 is -no_split; only a build uses it) |
| `-threads <n>` | threads in the pool that code generation and other parallel work share (default: one per core) |
| `-parse_threads <n>` | threads that parse files ahead of the checker (0 parses serially) |
| `-serial_parse` | parse every file on the checking thread |

##### Plugins

| Option | Meaning |
| --- | --- |
| `+Name` | run the metaprogram plugin module Name. Every argument after it, up to the next +Name, `-` or `--`, is that plugin's own option. Module parameters go in the name: '+Check(CHECK_BINDINGS=false)' (quote it in a shell). With -interp the program they compile is run. |
| `-` | the rest is for the metaprogram (`compiler_get_command_line`) |
| `--` | the rest is for the program (-interp only) |

##### Diagnostics and output

| Option | Meaning |
| --- | --- |
| `--color <when>` | colour in diagnostics: auto, always or never |
| `-A <code>` | allow a warning: print nothing (`warnings` for all, `leniencies` for the forms the official compiler rejects; `jacket explain` lists the codes) |
| `-W <code>` | warn about a code (the default) |
| `-D <code>` | deny a warning: print it as an error and fail (`-D warnings` for all) |
| `-no_color` | no colour in diagnostics |
| `-msvc_format` | one `path(line,col): error: message` line per diagnostic |
| `-quiet` | print nothing but errors (no build status either) |
| `-verbose` | say what is being done |
| `-timings` | print the wall time of each phase on stderr |
| `-h, -help, --help` | print this help |
| `-print_stdlib` | print the standard library folder jaic uses and exit |
| `-doc_comments <dir>` | print the doc comments of the library in &lt;dir&gt; as records, for `jacket test` |
| `-doc_gaps <dir>` | print which public declarations of the library in &lt;dir&gt; have docs and an example |
| `-doc_model <dir>` | print the documentation of the library in &lt;dir&gt; (items, signatures, docs) as JSON, for `jacket doc` |
| `-procedures <dirs>` | print every procedure with a body below the folders &lt;dirs&gt; (a path list) as records, for `jacket test --coverage`; `-os` and `-cpu` pick the `#if` branches |
| `-V, -version, --version` | print the version |
| `-profile` | count what the interpreter runs per procedure and print the busiest |
| `-profile_runs` | print the compile-time runs (#run, constants, metaprograms) by the work they did |
| `-run_budget <steps>` | steps (loop iterations and calls) one compile-time run may execute before it fails (default 1000000000; 0: no limit) |

##### Environment

| Variable | Meaning |
| --- | --- |
| `JAIC_MEMORY_LIMIT=<bytes\|nK\|nM\|nG>` | stop with exit status 120 once that much is allocated |
| `JAIC_DIAGNOSTICS=plain\|ascii\|unicode` | diagnostic layout (default: by terminal) |
| `NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE` | turn colour off or on |

exit status: 0 success, 1 the program failed to compile, an input file cannot be read or a runtime error stopped it (`-interp` otherwise exits with the program's own status), 2 a command-line mistake, 3 a build found no `main` to make an executable from, 120 the memory limit, 121 native code crashed under the interpreter, 101 an internal compiler error

### Creating a project

`jacket new` makes a directory with a `jacket.toml` and a `main.jai`; `jacket init` does the same in the current directory.

#### jacket new

Create a project in a new directory

```text
jacket new [OPTIONS] <NAME>
```

##### Arguments

| Argument | Meaning |
| --- | --- |
| `<NAME>` | Directory and project name |

##### Options

| Option | Meaning |
| --- | --- |
| `--lib` | A library (module.jai) instead of a program |
| `-h, --help` | Print help |

#### jacket init

Create a project in the current directory

```text
jacket init [OPTIONS]
```

##### Options

| Option | Meaning |
| --- | --- |
| `--lib` | A library (module.jai) instead of a program |
| `-h, --help` | Print help |

## Checking

### Linting

`jacket lint` runs `jailint` with the project's `[lint]` settings; its flags are those of `jailint`. [Lint rules](lint-rules.md) lists every rule, and [jailint](../tools/jailint.md) says how to configure and silence them.

Lints the Jai files under each path (default: the current directory).

```text
jailint [OPTIONS] [paths]...
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[paths]...` | Files, or folders that hold them (default: the current directory) |

#### Options

| Option | Meaning |
| --- | --- |
| `--fix` | Apply the fixes that are safe to apply automatically |
| `--config <file>` | Settings file (default: the nearest jacket.toml) |
| `-A <rule>` | Allow a rule (repeatable; `all` for every rule) |
| `-W <rule>` | Warn on a rule |
| `-D <rule>` | Deny a rule; `-D warnings` makes every warning an error |
| `--max-nesting <n>` | Blocks `excessive_nesting` allows (default: `[lint] max_nesting` in `jacket.toml`, else 6) |
| `--lifetime-mode <must\|strict>` | Lifetime proof mode (default: [lint.lifetimes] mode, else must) |
| `--compile-time-budget <steps>` | Steps (loop iterations and calls) one compile-time evaluation may run before it fails (default: 100000000; 0: no limit) |
| `--check-time-limit <seconds>` | Seconds each program may take to check before its check stops with partial results; every program gets the whole limit, and the run as a whole has none (default: 10; 0: no limit) |
| `--manifest <file>` | Write JSON source-check coverage and actual target contexts |
| `--context <program file>` | Check an enclosing program; report only the requested paths |
| `--checked-files <file>` | Write the actual checked paths, one per line |
| `--allow-incomplete` | Count programs that do not compile on this target without failing the run |
| `--list` | List the rules and their default levels |
| `-v, --verbose` | Show each program as it is compiled |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

#### Options shared with jaic

| Option | Meaning |
| --- | --- |
| `-os <os>` | target OS: linux, windows, macos or wasm |
| `-cpu <cpu>` | target CPU: x64 or arm64 |
| `-target <triple>` | build for an LLVM target triple (overrides -os and -cpu) |
| `-import_dir <dir>` | also look for modules in &lt;dir&gt; (repeatable; short form -I) |
| `-add "code"` | add the code to the target program (repeatable) |
| `-j, --jobs <n>` | run n compiles (or tests) at once (default: one per CPU) |
| `--color <when>` | colour in diagnostics: auto, always or never |

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage or configuration error.

### Formatting

`jacket fmt` runs `jaifmt` on the project with its `[format]` settings; its flags are those of `jaifmt`. [Format rules](../tools/jaifmt-rules.md) shows what each setting does.

Formats Jai source files in place. Directories are searched recursively for *.jai files.

Exit status: 0 success, 1 files would change (--check), 2 errors. With --is-ignored: 0 the file is ignored by the config's ignore list, 1 it is formatted, 2 errors.

```text
jaifmt [OPTIONS] [PATHS]...
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[PATHS]...` | Files or directories to format |

#### Options

| Option | Meaning |
| --- | --- |
| `--[no-]check` | Do not write; list the files that would change and exit with 1 |
| `--[no-]stdin` | Format standard input to standard output |
| `--range <START:END>` | With --stdin: format only the top-level statements the byte range START:END touches and copy the rest unchanged |
| `--config <FILE>` | Use this jacket.toml instead of the nearest one |
| `--is-ignored <FILE>` | Print nothing; exit 0 when FILE is ignored by the nearest config's ignore list, 1 when it is not (for editors) |
| `--set <KEY=VALUE>` | Set one [format] key, written as in jacket.toml (--set indent_width=2); implies --no-config. `jacket fmt` passes every resolved setting this way |
| `--[no-]no-config` | Read no jacket.toml: start from the defaults, then --config and --set. Nothing is ignored, since the caller chooses the files |
| `-v, --[no-]verbose` | Report every file, a summary, and lines over max_width |
| `--color <COLOR>` | Colour errors; auto means on a terminal, unless NO_COLOR is set [possible values: auto, always, never] |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

### Language server

`jacket lsp` runs `jailsp`, which editors start for diagnostics, completion and navigation.

```text
jailsp [--stdio] [--untrusted]
```

The Jai language server. An editor starts it and exchanges Language Server Protocol messages with it over stdin and stdout; it takes no files on the command line.

#### Options

| Option | Meaning |
| --- | --- |
| `--stdio` | talk over stdin and stdout (the default and only transport) |
| `--untrusted` | start without workspace trust: run no compile-time code (`#run`, build metaprograms, computed `#insert`) from the workspace until the client sends `jai/setTrusted` with `{"trusted": true}` |
| `--help, -h` | show this text |
| `--version, -V` | show the version |

To check a file from a terminal, use `jaic file.jai -check`; for lints, `jailint file.jai`.

### Explaining diagnostics

Every diagnostic carries a code; `jacket explain <code>` says what it means and how to fix it.

Explain a diagnostic code such as `type-mismatch`; without one, list them all

```text
jacket explain [CODE]
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[CODE]` | The code in `error[code]` |

#### Options

| Option | Meaning |
| --- | --- |
| `-h, --help` | Print help |

## Testing and documenting

### Testing

`jacket test` builds the `@test` procedures once and runs each in its own process. [jacket test](../tools/jacket-test.md) covers filters, coverage and the test library.

Build the tests once, then run each in its own process (`-os`/`-cpu` as in jaic; flags after `--` go to jaic)

```text
jacket test [OPTIONS] [FILTER]... [-- <FLAGS>...]
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[FILTER]...` | Run the tests whose name contains one of these; `test/subtest` runs one subtest |
| `[FLAGS]...` | Flags for jaic, after `--` |

#### Options

| Option | Meaning |
| --- | --- |
| `--exact` | Filters are whole names, not parts of names |
| `-E, --filter-expr <EXPR>` | Run the tests a filter expression selects, as `module(Sort) and not kind(property)`; with FILTERs, the tests both select (predicates: test, module, file, kind, all, none) |
| `--list` | Print the tests (and benchmarks) instead of running them |
| `--roots` | Print the test roots (the folders `[test] paths` expands to, narrowed by the current directory), one per line, and nothing else |
| `--deps` | With `--list`: print the source files the test programs read (what `jaic -deps` prints for each), instead of the tests |
| `--changed <FILE>` | Run only the tests a change to FILE can affect: those whose programs read it (as `--list --deps` shows), the tests of a root it adds an unlisted file to, and tests whose program does not compile. Repeat it for each changed file; it applies before `--partition` |
| `--coverage` | Report which lines of the code the tests ran (unit tests, golden programs and doc examples), and write them as lcov |
| `--coverage-out <FILE>` | Where `--coverage` writes its report [default: target/lcov.info, or target/coverage.html] |
| `--coverage-format <COVERAGE_FORMAT>` | The report's format Possible values: - lcov: lcov's text format, with line counts and procedures - html: One self-contained page: modules, files and the source with the count of each line [default: lcov] |
| `--coverage-by <COVERAGE_BY>` | What a row of the printed table is Possible values: - file - module: The nearest folder with a `module.jai` [default: file] |
| `--coverage-baseline <FILE>` | Fail when a module (or the total) covers fewer lines than this recorded baseline by more than `--coverage-margin` |
| `--coverage-margin <POINTS>` | Percentage points a module may lose against `--coverage-baseline` [default: 0.5] |
| `--coverage-record <FILE>` | Record this run's lines per module as a baseline |
| `--coverage-summary <FILE>` | Add a markdown table of the modules to this file (`$GITHUB_STEP_SUMMARY`) |
| `--coverage-include <DIR>` | Also report the code below this folder (repeatable); by default the test roots and the folders of `-import_dir` |
| `--watch` | Run again whenever a `.jai` file of the tests or the code they import changes |
| `-j, --jobs <N>` | Tests run at once, as jaic's -j [default: the machine's threads] |
| `--timeout <SECONDS>` | Seconds before a test without `@timeout` is killed; 0 for no limit [default: 120] |
| `--retries <N>` | Run a failed test again up to N times; one that then passes is FLAKY |
| `--fail-fast` | Start no more tests after the first failure |
| `--no-fail-fast` | Run every test even when a profile sets `fail_fast` |
| `--no-capture` | Show the tests' output as it is written (runs one test at a time) |
| `--no-isolation` | Run a root's tests in one process instead of one process each |
| `-q, --quiet` | Print only the tests that fail, time out or are flaky, then the summary |
| `--format <FORMAT>` | `human` text, or `json`: one event per line on standard output Possible values: - human - json:  One JSON object per event on standard output [default: human] |
| `--junit <FILE>` | Also write the results as `JUnit` XML |
| `--partition <SPEC>` | Run one part of the tests: `hash:K/N` (by name) or `count:K/N` (by position) |
| `--interp` | Run the tests in jaic's interpreter |
| `--os <OS>` | Target OS, as jaic's -os (also spelt `-os`) [possible values: linux, windows, macos, wasm] |
| `--cpu <CPU>` | Target CPU, as jaic's -cpu (also spelt `-cpu`) |
| `--runner <COMMAND>` | Command that runs test programs built for another target (`node tools/wasi_run.mjs`) |
| `--needs <LIST>` | Capabilities this machine has, for tests with `@needs` (`network,gpu`) |
| `--profile <NAME>` | Use `[test.profile.NAME]` from jacket.toml |
| `--path <DIR>` | Test the `tests/` of these folders instead of the project's |
| `--doc` | Run only the doc tests: the `jai` code blocks in a library's doc comments |
| `--check-only` | Only type-check the doc examples, without building or running them: with `--os`, the examples for that target from any machine |
| `--no-doc` | Leave the doc tests out |
| `--keep-temp` | Keep every test's temporary folder (failed tests keep theirs anyway) |
| `--accept` | Write what golden and compile-fail tests printed into their expected files, and keep the snapshots that differ or are new |
| `--review` | Go through the pending `.snap.new` snapshots one by one (only lists them without a terminal) |
| `--seed <N>` | The seed property tests start from [default: random for each run, shown with a failure] |
| `-h, --help` | Print help (see a summary with '-h') |

### Benchmarking

`jacket bench` builds the `@bench` procedures optimized and measures them. [jacket bench](../tools/jacket-bench.md) covers baselines and comparisons.

Build and measure the `@bench` procedures (optimized by default; the options of `test` select and build them)

```text
jacket bench [OPTIONS] [FILTER]... [-- <FLAGS>...]
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[FILTER]...` | Run the tests whose name contains one of these; `test/subtest` runs one subtest |
| `[FLAGS]...` | Flags for jaic, after `--` |

#### Options

| Option | Meaning |
| --- | --- |
| `--exact` | Filters are whole names, not parts of names |
| `-E, --filter-expr <EXPR>` | Run the tests a filter expression selects, as `module(Sort) and not kind(property)`; with FILTERs, the tests both select (predicates: test, module, file, kind, all, none) |
| `--list` | Print the tests (and benchmarks) instead of running them |
| `--roots` | Print the test roots (the folders `[test] paths` expands to, narrowed by the current directory), one per line, and nothing else |
| `--deps` | With `--list`: print the source files the test programs read (what `jaic -deps` prints for each), instead of the tests |
| `--changed <FILE>` | Run only the tests a change to FILE can affect: those whose programs read it (as `--list --deps` shows), the tests of a root it adds an unlisted file to, and tests whose program does not compile. Repeat it for each changed file; it applies before `--partition` |
| `--coverage` | Report which lines of the code the tests ran (unit tests, golden programs and doc examples), and write them as lcov |
| `--coverage-out <FILE>` | Where `--coverage` writes its report [default: target/lcov.info, or target/coverage.html] |
| `--coverage-format <COVERAGE_FORMAT>` | The report's format Possible values: - lcov: lcov's text format, with line counts and procedures - html: One self-contained page: modules, files and the source with the count of each line [default: lcov] |
| `--coverage-by <COVERAGE_BY>` | What a row of the printed table is Possible values: - file - module: The nearest folder with a `module.jai` [default: file] |
| `--coverage-baseline <FILE>` | Fail when a module (or the total) covers fewer lines than this recorded baseline by more than `--coverage-margin` |
| `--coverage-margin <POINTS>` | Percentage points a module may lose against `--coverage-baseline` [default: 0.5] |
| `--coverage-record <FILE>` | Record this run's lines per module as a baseline |
| `--coverage-summary <FILE>` | Add a markdown table of the modules to this file (`$GITHUB_STEP_SUMMARY`) |
| `--coverage-include <DIR>` | Also report the code below this folder (repeatable); by default the test roots and the folders of `-import_dir` |
| `--watch` | Run again whenever a `.jai` file of the tests or the code they import changes |
| `-j, --jobs <N>` | Tests run at once, as jaic's -j [default: the machine's threads] |
| `--timeout <SECONDS>` | Seconds before a test without `@timeout` is killed; 0 for no limit [default: 120] |
| `--retries <N>` | Run a failed test again up to N times; one that then passes is FLAKY |
| `--fail-fast` | Start no more tests after the first failure |
| `--no-fail-fast` | Run every test even when a profile sets `fail_fast` |
| `--no-capture` | Show the tests' output as it is written (runs one test at a time) |
| `--no-isolation` | Run a root's tests in one process instead of one process each |
| `-q, --quiet` | Print only the tests that fail, time out or are flaky, then the summary |
| `--format <FORMAT>` | `human` text, or `json`: one event per line on standard output Possible values: - human - json:  One JSON object per event on standard output [default: human] |
| `--junit <FILE>` | Also write the results as `JUnit` XML |
| `--partition <SPEC>` | Run one part of the tests: `hash:K/N` (by name) or `count:K/N` (by position) |
| `--interp` | Run the tests in jaic's interpreter |
| `--os <OS>` | Target OS, as jaic's -os (also spelt `-os`) [possible values: linux, windows, macos, wasm] |
| `--cpu <CPU>` | Target CPU, as jaic's -cpu (also spelt `-cpu`) |
| `--runner <COMMAND>` | Command that runs test programs built for another target (`node tools/wasi_run.mjs`) |
| `--needs <LIST>` | Capabilities this machine has, for tests with `@needs` (`network,gpu`) |
| `--profile <NAME>` | Use `[test.profile.NAME]` from jacket.toml |
| `--path <DIR>` | Test the `tests/` of these folders instead of the project's |
| `--doc` | Run only the doc tests: the `jai` code blocks in a library's doc comments |
| `--check-only` | Only type-check the doc examples, without building or running them: with `--os`, the examples for that target from any machine |
| `--no-doc` | Leave the doc tests out |
| `--keep-temp` | Keep every test's temporary folder (failed tests keep theirs anyway) |
| `--accept` | Write what golden and compile-fail tests printed into their expected files, and keep the snapshots that differ or are new |
| `--review` | Go through the pending `.snap.new` snapshots one by one (only lists them without a terminal) |
| `--seed <N>` | The seed property tests start from [default: random for each run, shown with a failure] |
| `--samples <N>` | Samples per benchmark, over `@samples` [default: 50] |
| `--warmup <MS>` | Warm-up per benchmark in milliseconds, at least; it goes on until the time per iteration is steady [default: 100] |
| `--measure-ms <MS>` | Time to spend sampling each benchmark in milliseconds, over `@min_time` [default: 1000] |
| `--resamples <N>` | Bootstrap resamples for the confidence intervals and the comparison [default: 100000] |
| `--noise-threshold <PCT>` | A change smaller than this many percent is noise [default: 2] |
| `--significance <P>` | A change counts when its p-value is below this [default: 0.05] |
| `--save-baseline <NAME>` | Write the results to `target/bench-baselines/NAME.json` |
| `--baseline <NAME>` | Compare with the results saved as NAME and show the change per benchmark |
| `--max-regression <PCT>` | Exit 1 when a benchmark regressed (the change is significant and beyond the noise threshold) by more than PCT percent |
| `-h, --help` | Print help (see a summary with '-h') |

### Documentation

`jacket doc` builds API documentation as a static website, like this one. [jacket doc](../tools/jacket-doc.md) covers the `[doc]` settings and themes.

Build the standard library's API documentation as a static website

```text
jacket doc [OPTIONS] [PATH]
```

#### Arguments

| Argument | Meaning |
| --- | --- |
| `[PATH]` | A library to document: a module (a folder with `module.jai`) or a folder of modules. Settings come from `[doc]` in its `jacket.toml` [default: `[doc] path` of the nearest `jacket.toml`, else the standard library] |

#### Options

| Option | Meaning |
| --- | --- |
| `--site-url <URL>` | Canonical site origin, over `[doc] url` |
| `--og-image <IMAGE>` | Social preview image URL or file, over `[doc] og_image` |
| `--out <DIR>` | Where to write the site [default: target/doc-site] |
| `--emit-input <DIR>` | Also write the resolved JSON view model (including with --check) |
| `--theme <DIR>` | Theme source overrides, over `[doc] theme`; relative to the current directory |
| `--timings` | Print time spent building models, views, compiling the theme, rendering and writing |
| `--base-url <PATH>` | The path the site is served under, with the slashes around it (`/docs/jacket/`) [default: /] |
| `--name <NAME>` | The name in the header and page titles, over `[doc] name` [default: the folder's name] |
| `--version <VERSION>` | The version shown in the header [default: none] |
| `--logo <FILE>` | An SVG or PNG for the header and the page icon, over `[doc] logo` [default: none] |
| `--repo <URL>` | The repository's web address, over `[doc] repo`; Source links go to `<URL>/blob/<ref>/<file>` [default: none, and no Source links] |
| `--runtime <DIR>` | A folder with `jai_wasm.wasm` and `engine.mjs` (a browser bundle) to copy into the site, so the Run buttons work; without it they are shown disabled |
| `--playground-url <URL>` | Where "open in playground" and the Playground link go, over `[doc] playground` [default: none, and no links] |
| `--source-ref <REF>` | The tag, branch or commit Source links point at [default: the checkout's commit] |
| `--module <NAME>` | Document only these modules (`String`, `Extensions/Http`); repeatable |
| `--check` | Build and check the links without writing the site or running the examples |
| `--no-test-examples` | Don't run the examples. By default each one with a Run button is run the way the browser runs it (WebAssembly target), and the build fails when one fails |
| `--allow-broken-links` | Report broken intra-doc links as warnings instead of failing |
| `-h, --help` | Print help |

## Setup

### Configuration

Settings live in the nearest `jacket.toml`; [jacket](../tools/jacket.md) lists its tables.

Show the configuration

```text
jacket config <COMMAND>
```

#### Commands

| Command | Meaning |
| --- | --- |
| `show` | Print the effective configuration |

#### Options

| Option | Meaning |
| --- | --- |
| `-h, --help` | Print help |

### Environment variables

Every environment variable `jaic`, `jacket`, `jailint` and the language server read. See [the environment registry](../tools/environment.md).

Path lists use the platform separator (`:` on Linux and macOS, `;` on Windows). An empty value counts as unset wherever a setting says so.

#### Settings

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `JACKET_CHECK_TIME_LIMIT` | Seconds one program check may take when the linter or language server runs compile-time code (`0`: no limit). CI sets it to `0`. | 10 seconds | jailint, jailsp |
| `JACKET_OUTPUT_DIR` | Relocates the build output directory (the `native-libs/` lookup and `jacket test --coverage` reports follow it). | `target/` in the project | jaic, jacket |
| `JACKET_TEST_DLLS` | A path list of directories whose `.dll` files `jacket test` links next to each Windows test executable. | unset | jacket |
| `JAIC_AR` | The program that archives object files into a static library. | the system `ar`, or `llvm-ar`, a MinGW `ar` or `lib.exe` where that is needed | jaic |
| `JAIC_BACKEND` | The code generation library (`libjaic_backend`) a release `jaic` opens when a build writes native code. | the code generation library next to the `jaic` executable | jaic |
| `JAIC_CACHE_DIR` | Where `jaic` unpacks the helpers it embeds (the crash runtime). | `$XDG_CACHE_HOME/jaic`, else the platform's user cache (`~/Library/Caches/jaic`, `~/.cache/jaic`) | jaic |
| `JAIC_CROSS_LIBS` | A path list of directories with libraries cross-built for a MinGW target; a cross build links from these instead of the host's native libraries. | unset | jaic |
| `JAIC_DIAGNOSTICS` | The diagnostic layout: `plain`, `ascii` or `unicode`. | by terminal: `unicode` or `ascii` on one, `plain` otherwise | jaic, jailint |
| `JAIC_DSYM` | When set, a macOS debug build writes a `.dSYM` bundle instead of keeping its object files. | unset: macOS debug builds keep their object files for the debugger | jaic |
| `JAIC_HOST_TWIN_DEBUG` | When set, the host twin prints what it generates. | unset | jaic |
| `JAIC_ID_SCRAMBLE` | The same as `JAIC_SYM_SCRAMBLE`, for AST ids. | unset | jaic |
| `JAIC_LINKER` | The program that links executables and libraries. Empty counts as unset. | the linker found for the target | jaic |
| `JAIC_MEMORY_LIMIT` | Stop with exit status 120 once this much memory is allocated: bytes, or a number with a `K`, `M` or `G` suffix. Child processes inherit it. | no limit | jaic |
| `JAIC_NATIVE_LIBS` | A path list of directories holding the prebuilt third-party native libraries (SDL3, FreeType, ...), replacing the default search. | `native-libs/<os>-<cpu>` beside the library, else under `target/` | jaic |
| `JAIC_NO_HOST_TWIN` | Any value but `0` turns the host twin off, which makes compile-time code with foreign calls fail the way it would without one. | unset: compile-time code that needs the host's types gets a host twin | jaic |
| `JAIC_PRELUDE` | The prelude file to load instead of the library's own, as a path (relative paths are taken from the library root). | the library's `prelude/module.jai`, else `Preload.jai` | jaic, jailint, jailsp |
| `JAIC_PROGRESS` | The build status lines: `off`, `plain` or `live`, whatever stderr is (`-quiet` still wins). | by terminal: a live status line on one, nothing otherwise | jaic, jacket |
| `JAIC_SANITIZER_CC` | The C driver that links a build made with a sanitizer. | the `clang` of the LLVM jaic was built with, else `cc` | jaic |
| `JAIC_STDLIB` | The standard library directory. `jacket test` also hands it to every `jaic` it starts. | the `stdlib/` next to the executable, else the checkout's | jaic, jacket, jailint, jailsp, tests |
| `JAIC_SYM_SCRAMBLE` | A seed that spreads the interner's symbol numbers apart, so output that depends on them rather than spellings changes (`tools/differential/determinism.py`). | unset | jaic |
| `JAIC_TCPLISTEN` | Comma-separated `host:port` addresses a `-interp -os wasm` run listens on. Each becomes a pre-opened socket the program accepts connections from, numbered 3, 4, ... in order, and `LISTEN_FDS` holds how many there are (the convention of wasmtime's `--tcplisten`). | unset: a `-interp -os wasm` run has no sockets | jaic |
| `JAIC_WASM_LD` | The `wasm-ld` that links WebAssembly output. | `wasm-ld` next to the build's LLVM, an LLD install, then `PATH` | jaic |
| `JAILINT_LIFETIME_MEMORY` | With the lifetime_memory example and one worker, measure selected lifetime facts separately from all other lint facts after each compiler finishes linting. | No destructive retained-fact profiling. | jailint, tests |
| `JAILINT_LIFETIME_STATS` | Print per-file body, CFG, candidate and rejection counts for the lifetime rules. | No lifetime coverage counters. | jailint |

#### Set by one tool for another

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `JACKET_PROJECT` | The project name `jaic` shows in its `Compiling` line; `jacket` sets it. | the main file's name | jaic |
| `JAIC_COVERAGE_DIR` | The directory a `-interp` run writes its line coverage to, one file per program. `jacket test --coverage` sets it. | the working directory | jaic |
| `JAIC_COVERAGE_FILE` | The file a `-interp` run writes its line coverage to. | unset: the file is named for the program, in `JAIC_COVERAGE_DIR` | jaic |
| `JAIC_REPORT_EXE` | A file `jaic` appends the absolute path of each executable it writes to. `jacket run` and `jacket test` set it, because a metaprogram picks the name. | unset | jaic |

#### Tests and fuzzing

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `EXPECTED_PRELUDE` | The prelude the child process of the prelude-selection tests must find, or `missing`. | unset | tests |
| `JACKET_DATABASE_URL` | A PostgreSQL connection URL for the typed-query tests. | unset: the Postgres tests skip | tests |
| `JACKET_STDLIB_TARGETS_PART` | `K/N`: the stdlib-by-target check takes every Nth module and target pair from the Kth, so CI can split it. | `1/1` | tests |
| `JACKET_TEST_CODEGEN_UNITS` | Passes `-codegen_units N` to the native build tests, to run them split into several modules. | unset | tests |
| `JAIC_ABI_CLANG` | The `clang` the C ABI tests compare against. | `clang` | tests |
| `JAIC_ABI_CROSS` | Cross targets the C ABI tests also compile for, as a comma list of specs. | unset | tests |
| `JAIC_CORPUS_ROOT` | The checkout whose `corpus/upstream` and `stdlib` the corpus sweep parses. | the repository | tests |
| `JAIC_CORPUS_STAGE` | How far the corpus sweep takes each file. | `parse` | tests |
| `JAIC_KEEP_TEST_TMP` | When set, test scratch directories are kept after the test. | unset: scratch directories are removed | tests |
| `JAIC_PRELUDE_TEST_CHILD` | Marks the child process of the prelude-selection tests, which run themselves again with `JAIC_PRELUDE` set. | unset | tests |
| `JAIC_REQUIRE_TOOLCHAIN` | `1`, or a comma list of tool names: a missing tool fails the test instead of skipping it. Always on under `CI`. | unset: a test whose tool is missing skips | tests |
| `JAIC_SPECULATIVE_BODIES` | When set, checks queued bodies speculatively first, like `-dev_speculative_bodies` (only in a `jaic` built with the `speculative-bodies` feature). | unset | jaic, tests |
| `JAIC_TEST_FONT_PATH` | A font file for stdlib tests that render text; without it they skip locally and fail under `CI`. | unset | jacket |
| `JAIC_TEST_LIBRARY_MARKER` | A file the native-link test's library writes to, to show it was loaded. | unset | tests |
| `JAIC_TEST_TMPDIR` | Where test scratch directories are made. | the system temporary directory | tests |
| `JAIC_WASM_TEST` | A value the WebAssembly test passes to the program, which prints it back. | unset | tests |
| `JAILSP_ENFORCE_SCALE_BUDGET` | `1`: the language server scale test fails when a case is over its budget. | unset: the budget is only reported | tests |
| `JAI_FUZZ_VERBOSE` | When set, the fuzz harness prints each input it checks. | unset | fuzzing |
| `JAI_IDE_AUDIT_REVISION` | Read at build time: the revision the language server coverage audit stamps into its report. | unset | tests |
| `TYPING_FUZZ_DUMP` | A directory the typing-session rig writes the failing document to. | unset | tests |

#### Standard variables

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `ANSICON` | Set by ANSICON; marks a Windows console that takes ANSI control codes. | unset | jaic, jacket |
| `CC` | The C compiler the native tests build their helper libraries with. | `cc` | tests |
| `CI` | Set by CI services: the build status lines are plain, and a missing test tool fails instead of skipping. | unset | jaic, jacket, tests |
| `CLICOLOR_FORCE` | The same as `FORCE_COLOR`. | unset | jaic, jailint |
| `COLUMNS` | The width the live status line is fitted to. | the terminal's width | jaic |
| `ConEmuANSI` | `ON` marks a Windows console that takes ANSI control codes. | unset | jaic, jacket, jailint |
| `DEVELOPER_DIR` | The Xcode installation whose SDK and libclang jaic uses. | `xcode-select -p` | jaic |
| `FORCE_COLOR` | Anything but `0` turns colour on even when output is not a terminal. | unset | jaic, jailint |
| `GITHUB_ACTIONS` | Set by GitHub Actions: tests that need a capable filesystem stop skipping. | unset | jacket, tests |
| `GITHUB_TOKEN` | Sent with `jacket upgrade`'s release lookups, which raises GitHub's rate limit. | unset | jacket |
| `HOME` | Locates the user cache when neither `JAIC_CACHE_DIR` nor `XDG_CACHE_HOME` is set. | unset | jaic, jacket |
| `INSTA_UPDATE` | `always` rewrites the test snapshots instead of failing on a difference. | unset | tests |
| `LANG` | See `LC_ALL`. | unset | jaic, jailint |
| `LC_ALL` | With `LC_CTYPE` and `LANG`: a UTF-8 locale selects the Unicode diagnostic layout. | unset | jaic, jailint |
| `LC_CTYPE` | See `LC_ALL`. | unset | jaic, jailint |
| `LIB` | On Windows, the library directories the MSVC linker searches; set by a developer prompt. | found from the Visual Studio and Windows SDK installs | jaic |
| `LIBRARY_PATH` | Extra library directories compile-time code may load native libraries from on Windows. | unset | jaic |
| `LLVM_PREFIX` | The LLVM install to use when the `llvm-sys` variable above is unset. CI sets both. | `llvm-config --prefix`, then Homebrew's `llvm@<major>` or `/usr/lib/llvm-<major>` | jaic, jacket |
| `LLVM_SYS_231_PREFIX` | The LLVM install jaic is built against; its `bin/` supplies `clang`, `wasm-ld` and the LLVM tools. The name carries the LLVM version (`LLVM_SYS_<major><minor>_PREFIX`). | unset | jaic, jacket |
| `LOCALAPPDATA` | The Windows user cache root; jacket caches its compiled documentation theme here. | unset | jacket |
| `MACOSX_DEPLOYMENT_TARGET` | The macOS version native output targets. When it is set, jaic links through `cc` instead of calling `ld` itself. | the SDK's | jaic |
| `NO_COLOR` | Any value turns colour off. | unset | jaic, jacket, jailint |
| `PATH` | Where tools such as the linker, `clang` and `wasm-ld` are searched for. | the process's | jaic, jacket, jailsp, tests |
| `PATHEXT` | The executable extensions tried when a test looks a tool up on Windows. | the Windows default | tests |
| `ProgramFiles` | Where Visual Studio and the Windows SDK are looked for. | `C:\Program Files` | jaic |
| `ProgramFiles(x86)` | See `ProgramFiles`. | `C:\Program Files (x86)` | jaic |
| `RUST_BACKTRACE` | When set, an internal compiler error shows the Rust backtrace. | unset | jaic |
| `SDKROOT` | The macOS SDK `cc` links against. When it is set, jaic links through `cc` instead of calling `ld` itself. | unset | jaic |
| `SystemRoot` | Locates the Windows system tools `jacket upgrade` uses. | `C:\Windows` | jacket |
| `TERM` | `dumb` turns colour, layout characters and the live status line off. | unset | jaic, jacket, jailint |
| `WT_SESSION` | Set by Windows Terminal; it gets the Unicode diagnostic layout. | unset | jaic, jailint |
| `XDG_CACHE_HOME` | The user cache root; `jaic` uses its `jaic` folder. | unset | jaic, jacket |

### Installing

`jacket version` prints the version of `jacket` and every tool it fronts; `jacket upgrade` installs the latest release over this one.

#### jacket version

Print the version of jacket and of every tool it fronts

```text
jacket version
```

##### Options

| Option | Meaning |
| --- | --- |
| `-h, --help` | Print help |

#### jacket upgrade

Install the latest release over this installation

```text
jacket upgrade [OPTIONS]
```

##### Options

| Option | Meaning |
| --- | --- |
| `--version <TAG>` | Install this release (`0.22.0` or `v0.22.0`) instead of the latest |
| `--check` | Only report whether an update exists |
| `-y, --yes` | Do not ask for confirmation |
| `-h, --help` | Print help |
