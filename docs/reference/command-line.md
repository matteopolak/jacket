# Command line

Every flag of `jaic`, `jailint` and `jacket`, as their `--help` prints it. [The `jaic` command line](../tools/cli.md), [jailint](../tools/jailint.md) and [jacket](../tools/jacket.md) say how the flags fit together.

## jaic

```text
jaic: compile, check and run Jai programs

usage: jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]

By default jaic builds a native executable. Options may come before or after the file.

modes:
  -check                    compile the program without running or writing anything
  -interp                   run `main` in the interpreter (arguments after `--` go to the program)
  -deps                     like -check, then print every source file the compile read, one per line

program:
  -add "code"               add the code to the target program (repeatable)
  -run "code"               run the code at compile time, as a `#run` (repeatable)
  -import_dir <dir>         also look for modules in <dir> (repeatable; short form -I)
  -os <os>                  target OS: linux, windows, macos or wasm
  -cpu <cpu>                target CPU: x64 or arm64
  -target <triple>          build for an LLVM target triple (overrides -os and -cpu)
  -no_dce                   type-check unreferenced module code too
  -no_check                 accepted and ignored: jaic does not run the Check plugin unless asked
  -no_check_bindings        set CHECK_BINDINGS=false on a +Check plugin, if there is one

output (building only):
  -j, --jobs <n>            run n compiles (or tests) at once (default: one per CPU)
  -exe <name>               the executable's name; it may include a directory and extension
                            (`-exe build/app.exe`). A bare name goes in the current directory
  -output_path <dir>        the directory the executable is written to
  -o, -optimized            optimized build (LLVM O2) without stack traces or runtime checks
  -od, -optimized_debug     lightly optimized build (LLVM O1) that keeps stack traces and checks
  -very_debug               unoptimized build with every runtime check on
  -O0, -O1, -O2, -O3        pick the LLVM optimization level directly
  -no_inline                do not inline procedures
  -no_split                 generate code as one module instead of several
  -no_backtrace_on_crash    leave the crash handler out of the program
  -emit_ir <file.ll>        also write the LLVM IR
  -no_debug_info            leave out native debug information
  -sanitize <list>          address, undefined and/or fuzzer, comma-separated
  -coverage                 record line and function coverage (docs/tools/coverage.md)
  -no_workspace_output      (-interp only) do not write what a metaprogram's workspaces ask for
  -codegen_units <n>        generate code as exactly <n> modules (1 is -no_split; only a build uses it)
  -threads <n>              threads in the pool that code generation and other parallel work share (default: one per core)
  -parse_threads <n>        threads that parse files ahead of the checker (0 parses serially)
  -serial_parse             parse every file on the checking thread

plugins:
  +Name                     run the metaprogram plugin module Name. Every argument after it, up
                            to the next +Name, `-` or `--`, is that plugin's own option.
                            Module parameters go in the name: '+Check(CHECK_BINDINGS=false)'
                            (quote it in a shell). With -interp the program they compile is run.
  -                         the rest is for the metaprogram (`compiler_get_command_line`)
  --                        the rest is for the program (-interp only)

diagnostics and output:
  --color <when>            colour in diagnostics: auto, always or never
  -A <code>                 allow a warning: print nothing (`warnings` for all, `leniencies` for the forms the
                            official compiler rejects; `jacket explain` lists the codes)
  -W <code>                 warn about a code (the default)
  -D <code>                 deny a warning: print it as an error and fail (`-D warnings` for all)
  -no_color                 no colour in diagnostics
  -msvc_format              one `path(line,col): error: message` line per diagnostic
  -quiet                    print nothing but errors (no build status either)
  -verbose                  say what is being done
  -timings                  print the wall time of each phase on stderr
  -h, -help, --help         print this help
  -print_stdlib             print the standard library folder jaic uses and exit
  -doc_comments <dir>       print the doc comments of the library in <dir> as records, for `jacket test`
  -doc_gaps <dir>           print which public declarations of the library in <dir> have docs and an example
  -doc_model <dir>          print the documentation of the library in <dir> (items, signatures, docs) as JSON,
                            for `jacket doc`
  -procedures <dirs>        print every procedure with a body below the folders <dirs> (a path list) as records,
                            for `jacket test --coverage`; `-os` and `-cpu` pick the `#if` branches
  -V, -version, --version   print the version
  -profile                  count what the interpreter runs per procedure and print the busiest
  -profile_runs             print the compile-time runs (#run, constants, metaprograms) by the work they did
  -run_budget <steps>       steps (loop iterations and calls) one compile-time run may execute before it fails (default 1000000000; 0: no limit)

environment:
  JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>  stop with exit status 120 once that much is allocated
  JAIC_DIAGNOSTICS=plain|ascii|unicode  diagnostic layout (default: by terminal)
  NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE  turn colour off or on

exit status: 0 success, 1 the program failed to compile, an input file cannot be read
or a runtime error stopped it (`-interp` otherwise exits with the program's own status),
2 a command-line mistake, 3 a build found no `main` to make an executable from,
120 the memory limit, 121 native code crashed under the interpreter, 101 an
internal compiler error
```

## jailint

```text
Lints the Jai files under each path (default: the current directory).

Usage: jailint [OPTIONS] [paths]...

Arguments:
  [paths]...  Files, or folders that hold them (default: the current directory)

Options:
      --fix                          Apply the fixes that are safe to apply automatically
      --config <file>                Settings file (default: the nearest jacket.toml)
  -A <rule>                          Allow a rule (repeatable; `all` for every rule)
  -W <rule>                          Warn on a rule
  -D <rule>                          Deny a rule; `-D warnings` makes every warning an error
      --max-nesting <n>              Blocks `excessive_nesting` allows (default: `[lint] max_nesting` in `jacket.toml`, else 6)
      --compile-time-budget <steps>  Steps (loop iterations and calls) one compile-time evaluation may run before it fails (default: 100000000; 0: no limit)
      --check-time-limit <seconds>   Seconds each program may take to check before its check stops with partial results; every program gets the whole limit, and the run as a whole has none (default: 10; 0: no limit)
      --manifest <file>              Write JSON source-check coverage and actual target contexts
      --context <program file>       Check an enclosing program; report only the requested paths
      --checked-files <file>         Write the actual checked paths, one per line
      --allow-incomplete             Count programs that do not compile on this target without failing the run
      --list                         List the rules and their default levels
  -v, --verbose                      Show each program as it is compiled
  -h, --help                         Print help
  -V, --version                      Print version

Options shared with jaic:
  -os <os>                      target OS: linux, windows, macos or wasm
  -cpu <cpu>                    target CPU: x64 or arm64
  -target <triple>              build for an LLVM target triple (overrides -os and -cpu)
  -import_dir <dir>             also look for modules in <dir> (repeatable; short form -I)
  -add "code"                   add the code to the target program (repeatable)
  -j, --jobs <n>                run n compiles (or tests) at once (default: one per CPU)
  --color <when>                colour in diagnostics: auto, always or never

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a
usage or configuration error.
```

## jacket

```text
One front door to the Jai toolchain: compiler, linter, formatter and language server.

Usage: jacket <COMMAND>

Commands:
  build    Compile a program: `jaic <args>`, flags untouched
  check    Type-check without writing anything: `jaic -check <args>`
  run      Build, then run the program; arguments after `--` go to it (`-interp` runs it in jaic)
  lint     Lint with `jailint` (`--fix`, `-D warnings`, paths)
  fmt      Format with `jaifmt` (`--check`, paths; the project by default)
  lsp      Run the language server over stdio (`jailsp`)
  new      Create a project in a new directory
  init     Create a project in the current directory
  test     Build the tests once, then run each in its own process (`-os`/`-cpu` as in jaic; flags after `--` go to jaic)
  bench    Build and measure the `@bench` procedures (optimized by default; the options of `test` select and build them)
  doc      Build the standard library's API documentation as a static website
  config   Show the configuration
  clean    Remove the executable `jacket build` writes
  explain  Explain a diagnostic code such as `type-mismatch`; without one, list them all
  version  Print the version of jacket and of every tool it fronts
  upgrade  Install the latest release over this installation
  jaic     Run `jaic` with exactly these arguments
  jailint  Run `jailint` with exactly these arguments
  jailsp   Run `jailsp` with exactly these arguments
  jaifmt   Run `jaifmt` with exactly these arguments

Options:
  -h, --help     Print help
  -V, --version  Print version

`jacket` is a drop-in for the tools it fronts. `build`, `check` and `run` take the flags of `jaic`
(single-dash, as in `jacket build -o -exe app main.jai`) and pass them through untouched; `lint`,
`fmt` and `lsp` take those of `jailint`, `jaifmt` and `jailsp`. `jacket build -help` prints the
compiler's own help. What `jacket` adds is the project: with no .jai file named, `build` and
`check` use `[project] entry` from the nearest jacket.toml (else build.jai, first.jai, main.jai,
src/main.jai or module.jai), and `[experimental]` settings become jaic flags unless you pass them.

A command that is none of these runs `jacket-<command>` from PATH, as `cargo` does.

Configuration is the nearest jacket.toml (docs/tools/jacket.md).
```

### jacket new

```text
Create a project in a new directory

Usage: jacket new [OPTIONS] <NAME>

Arguments:
  <NAME>  Directory and project name

Options:
      --lib   A library (module.jai) instead of a program
  -h, --help  Print help
```

### jacket init

```text
Create a project in the current directory

Usage: jacket init [OPTIONS]

Options:
      --lib   A library (module.jai) instead of a program
  -h, --help  Print help
```

### jacket test

```text
Build the tests once, then run each in its own process (`-os`/`-cpu` as in jaic; flags after `--` go to jaic)

Usage: jacket test [OPTIONS] [FILTER]... [-- <FLAGS>...]

Arguments:
  [FILTER]...
          Run the tests whose name contains one of these; `test/subtest` runs one subtest

  [FLAGS]...
          Flags for jaic, after `--`

Options:
      --exact
          Filters are whole names, not parts of names

  -E, --filter-expr <EXPR>
          Run the tests a filter expression selects, as `module(Sort) and not kind(property)`; with FILTERs, the tests both select (predicates: test, module, file, kind, all, none)

      --list
          Print the tests (and benchmarks) instead of running them

      --deps
          With `--list`: print the source files the test programs read (what `jaic -deps` prints for each), instead of the tests

      --coverage
          Report which lines of the code the tests ran (unit tests, golden programs and doc examples), and write them as lcov

      --coverage-out <FILE>
          Where `--coverage` writes its report [default: target/lcov.info, or target/coverage.html]

      --coverage-format <COVERAGE_FORMAT>
          The report's format

          Possible values:
          - lcov: lcov's text format, with line counts and procedures
          - html: One self-contained page: modules, files and the source with the count of each line

          [default: lcov]

      --coverage-by <COVERAGE_BY>
          What a row of the printed table is

          Possible values:
          - file
          - module: The nearest folder with a `module.jai`

          [default: file]

      --coverage-baseline <FILE>
          Fail when a module (or the total) covers fewer lines than this recorded baseline by more than `--coverage-margin`

      --coverage-margin <POINTS>
          Percentage points a module may lose against `--coverage-baseline`

          [default: 0.5]

      --coverage-record <FILE>
          Record this run's lines per module as a baseline

      --coverage-summary <FILE>
          Add a markdown table of the modules to this file (`$GITHUB_STEP_SUMMARY`)

      --coverage-include <DIR>
          Also report the code below this folder (repeatable); by default the test roots and the folders of `-import_dir`

      --watch
          Run again whenever a `.jai` file of the tests or the code they import changes

  -j, --jobs <N>
          Tests run at once, as jaic's -j [default: the machine's threads]

      --timeout <SECONDS>
          Seconds before a test without `@timeout` is killed; 0 for no limit [default: 120]

      --retries <N>
          Run a failed test again up to N times; one that then passes is FLAKY

      --fail-fast
          Start no more tests after the first failure

      --no-fail-fast
          Run every test even when a profile sets `fail_fast`

      --no-capture
          Show the tests' output as it is written (runs one test at a time)

      --no-isolation
          Run a root's tests in one process instead of one process each

  -q, --quiet
          Print only the tests that fail, time out or are flaky, then the summary

      --format <FORMAT>
          `human` text, or `json`: one event per line on standard output

          Possible values:
          - human
          - json:  One JSON object per event on standard output

          [default: human]

      --junit <FILE>
          Also write the results as `JUnit` XML

      --partition <SPEC>
          Run one part of the tests: `hash:K/N` (by name) or `count:K/N` (by position)

      --interp
          Run the tests in jaic's interpreter

      --os <OS>
          Target OS, as jaic's -os (also spelt `-os`)

          [possible values: linux, windows, macos, wasm]

      --cpu <CPU>
          Target CPU, as jaic's -cpu (also spelt `-cpu`)

      --runner <COMMAND>
          Command that runs test programs built for another target (`node tools/wasi_run.mjs`)

      --needs <LIST>
          Capabilities this machine has, for tests with `@needs` (`network,gpu`)

      --profile <NAME>
          Use `[test.profile.NAME]` from jacket.toml

      --path <DIR>
          Test the `tests/` of these folders instead of the project's

      --doc
          Run only the doc tests: the `jai` code blocks in a library's doc comments

      --no-doc
          Leave the doc tests out

      --keep-temp
          Keep every test's temporary folder (failed tests keep theirs anyway)

      --accept
          Write what golden and compile-fail tests printed into their expected files, and keep the snapshots that differ or are new

      --review
          Go through the pending `.snap.new` snapshots one by one (only lists them without a terminal)

      --seed <N>
          The seed property tests start from [default: random for each run, shown with a failure]

  -h, --help
          Print help (see a summary with '-h')
```

### jacket bench

```text
Build and measure the `@bench` procedures (optimized by default; the options of `test` select and build them)

Usage: jacket bench [OPTIONS] [FILTER]... [-- <FLAGS>...]

Arguments:
  [FILTER]...
          Run the tests whose name contains one of these; `test/subtest` runs one subtest

  [FLAGS]...
          Flags for jaic, after `--`

Options:
      --exact
          Filters are whole names, not parts of names

  -E, --filter-expr <EXPR>
          Run the tests a filter expression selects, as `module(Sort) and not kind(property)`; with FILTERs, the tests both select (predicates: test, module, file, kind, all, none)

      --list
          Print the tests (and benchmarks) instead of running them

      --deps
          With `--list`: print the source files the test programs read (what `jaic -deps` prints for each), instead of the tests

      --coverage
          Report which lines of the code the tests ran (unit tests, golden programs and doc examples), and write them as lcov

      --coverage-out <FILE>
          Where `--coverage` writes its report [default: target/lcov.info, or target/coverage.html]

      --coverage-format <COVERAGE_FORMAT>
          The report's format

          Possible values:
          - lcov: lcov's text format, with line counts and procedures
          - html: One self-contained page: modules, files and the source with the count of each line

          [default: lcov]

      --coverage-by <COVERAGE_BY>
          What a row of the printed table is

          Possible values:
          - file
          - module: The nearest folder with a `module.jai`

          [default: file]

      --coverage-baseline <FILE>
          Fail when a module (or the total) covers fewer lines than this recorded baseline by more than `--coverage-margin`

      --coverage-margin <POINTS>
          Percentage points a module may lose against `--coverage-baseline`

          [default: 0.5]

      --coverage-record <FILE>
          Record this run's lines per module as a baseline

      --coverage-summary <FILE>
          Add a markdown table of the modules to this file (`$GITHUB_STEP_SUMMARY`)

      --coverage-include <DIR>
          Also report the code below this folder (repeatable); by default the test roots and the folders of `-import_dir`

      --watch
          Run again whenever a `.jai` file of the tests or the code they import changes

  -j, --jobs <N>
          Tests run at once, as jaic's -j [default: the machine's threads]

      --timeout <SECONDS>
          Seconds before a test without `@timeout` is killed; 0 for no limit [default: 120]

      --retries <N>
          Run a failed test again up to N times; one that then passes is FLAKY

      --fail-fast
          Start no more tests after the first failure

      --no-fail-fast
          Run every test even when a profile sets `fail_fast`

      --no-capture
          Show the tests' output as it is written (runs one test at a time)

      --no-isolation
          Run a root's tests in one process instead of one process each

  -q, --quiet
          Print only the tests that fail, time out or are flaky, then the summary

      --format <FORMAT>
          `human` text, or `json`: one event per line on standard output

          Possible values:
          - human
          - json:  One JSON object per event on standard output

          [default: human]

      --junit <FILE>
          Also write the results as `JUnit` XML

      --partition <SPEC>
          Run one part of the tests: `hash:K/N` (by name) or `count:K/N` (by position)

      --interp
          Run the tests in jaic's interpreter

      --os <OS>
          Target OS, as jaic's -os (also spelt `-os`)

          [possible values: linux, windows, macos, wasm]

      --cpu <CPU>
          Target CPU, as jaic's -cpu (also spelt `-cpu`)

      --runner <COMMAND>
          Command that runs test programs built for another target (`node tools/wasi_run.mjs`)

      --needs <LIST>
          Capabilities this machine has, for tests with `@needs` (`network,gpu`)

      --profile <NAME>
          Use `[test.profile.NAME]` from jacket.toml

      --path <DIR>
          Test the `tests/` of these folders instead of the project's

      --doc
          Run only the doc tests: the `jai` code blocks in a library's doc comments

      --no-doc
          Leave the doc tests out

      --keep-temp
          Keep every test's temporary folder (failed tests keep theirs anyway)

      --accept
          Write what golden and compile-fail tests printed into their expected files, and keep the snapshots that differ or are new

      --review
          Go through the pending `.snap.new` snapshots one by one (only lists them without a terminal)

      --seed <N>
          The seed property tests start from [default: random for each run, shown with a failure]

      --samples <N>
          Samples per benchmark, over `@samples` [default: 50]

      --warmup <MS>
          Warm-up per benchmark in milliseconds, at least; it goes on until the time per iteration is steady [default: 100]

      --measure-ms <MS>
          Time to spend sampling each benchmark in milliseconds, over `@min_time` [default: 1000]

      --resamples <N>
          Bootstrap resamples for the confidence intervals and the comparison

          [default: 100000]

      --noise-threshold <PCT>
          A change smaller than this many percent is noise

          [default: 2]

      --significance <P>
          A change counts when its p-value is below this

          [default: 0.05]

      --save-baseline <NAME>
          Write the results to `target/bench-baselines/NAME.json`

      --baseline <NAME>
          Compare with the results saved as NAME and show the change per benchmark

      --max-regression <PCT>
          Exit 1 when a benchmark regressed (the change is significant and beyond the noise threshold) by more than PCT percent

  -h, --help
          Print help (see a summary with '-h')
```

### jacket doc

```text
Build the standard library's API documentation as a static website

Usage: jacket doc [OPTIONS]

Options:
      --out <DIR>             Where to write the site [default: target/doc-site]
      --base-url <PATH>       The path the site is served under, with the slashes around it (`/docs/jacket/`) [default: /]
      --runtime <DIR>         A folder with `jai_wasm.wasm` and `engine.mjs` (a browser bundle) to copy into the site, so the Run buttons work; without it they are shown disabled
      --playground-url <URL>  Where "open in playground" and the Playground link go [default: https://matteopolak.com/playground/jacket]
      --source-ref <REF>      The tag or branch Source links point at [default: v<this version>]
      --module <NAME>         Document only these modules (`String`, `Extensions/Http`); repeatable
      --check                 Build and check the links without writing the site
      --allow-broken-links    Report broken intra-doc links as warnings instead of failing
  -h, --help                  Print help
```

### jacket config

```text
Show the configuration

Usage: jacket config <COMMAND>

Commands:
  show  Print the effective configuration

Options:
  -h, --help  Print help
```

### jacket clean

```text
Remove the executable `jacket build` writes

Usage: jacket clean

Options:
  -h, --help  Print help
```

### jacket explain

```text
Explain a diagnostic code such as `type-mismatch`; without one, list them all

Usage: jacket explain [CODE]

Arguments:
  [CODE]  The code in `error[code]`

Options:
  -h, --help  Print help
```

### jacket version

```text
Print the version of jacket and of every tool it fronts

Usage: jacket version

Options:
  -h, --help  Print help
```

### jacket upgrade

```text
Install the latest release over this installation

Usage: jacket upgrade [OPTIONS]

Options:
      --version <TAG>  Install this release (`0.22.0` or `v0.22.0`) instead of the latest
      --check          Only report whether an update exists
  -y, --yes            Do not ask for confirmation
  -h, --help           Print help
```
