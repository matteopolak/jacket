# The `jaic` command line

`jaic` is the compiler. One command, no subcommands: give it a `.jai` file and it builds a native executable next to it.

```sh
jaic main.jai                  # build ./main (main.exe on Windows)
jaic main.jai -check           # only type-check; write nothing
jaic main.jai -interp          # run main in the interpreter, no executable
jaic main.jai -interp -- a b   # ...passing arguments to the program
jaic main.jai -o               # optimized build
jaic main.jai -os windows -cpu arm64
```

Options go before or after the file. `jaic -help` lists every option; the full table is the [command-line reference](../reference/command-line.md). [`jacket`](jacket.md) runs `jaic` with your arguments plus the project's entry file and settings. Other source files come in through `#load` and `#import`.

## Options worth knowing

- `-check` and `-interp` can't be combined. Build-only options (`-exe`, `-o`, `-sanitize`, ...) are accepted and ignored with either, so one command line can build, check or interpret.
- `-deps` type-checks and prints the absolute path of every source file read, sorted. CI uses it to pick jobs ([CI job selection](ci-job-selection.md)).
- `-print_stdlib` prints the standard library folder in use.
- `-add "code"` adds code to the program; `-run "code"` runs it at compile time. `-I dir` adds a module folder. `-j <n>` sets worker threads.
- `-target <triple>` builds for an LLVM triple that `-os`/`-cpu` don't cover.
- `-no_dce` also type-checks module code nothing references.
- `-run_budget N` caps the steps one compile-time run may take (default 1000000000, `0` for no limit), so a `#run` that never ends can't hang a build.
- `-profile`, `-profile_ops` and `-coverage` (with `-interp`) count what the interpreter runs ([coverage](coverage.md)).
- `-dev_*` flags study the compiler itself and may change at any time.

### Build flavors

With no flavor you get an unoptimized build with stack traces and runtime checks. `-od` keeps the checks and lightly optimizes, `-o` optimizes fully and drops stack traces and checks, `-very_debug` turns every check on. `-O0` to `-O3` pick the LLVM level directly; don't combine them with a flavor. `-no_inline`, `-no_split`, `-no_debug_info` and `-no_backtrace_on_crash` each turn off one thing. `-sanitize address,undefined,fuzzer` builds with sanitizers, `-emit_ir out.ll` also writes LLVM IR. `-codegen_units N`, `-threads N` and `-parse_threads N` tune parallelism. These override a build metaprogram's optimization level.

### Where the executable goes

Named after the source file, next to it (`.exe` on Windows). `-exe app` changes the name, `-exe build/app` is relative to where you ran `jaic`, and `-output_path out/` sets the directory a relative `-exe` goes in. Only the main workspace is affected.

### Arguments for plugins, metaprograms and programs

```sh
jaic build.jai +Check - release -- arg1 arg2
#              ^^^^^^ ^^^^^^^^^ ^^^^^^^^^^^^
#              plugin   meta-     program
#                      program   (with -interp)
```

- `+Name` loads the plugin `Name`; what follows, up to the next `+Name`, `-` or `--`, is its options. Module parameters go in the name: `'+Check(CHECK_BINDINGS=false)'`.
- A lone `-` starts the metaprogram's arguments (`compiler_get_command_line`).
- `--` starts the program's arguments. Only `-interp` runs the program, so `--` otherwise is an error.

## Output

On a terminal `jaic` prints a `Compiling` line, a live status line (`Parsing`, `Checking`, `Running #run`, `Generating`, `Optimizing`, `Linking`) that appears after 150 ms, and a `Finished` line. When stderr is not a terminal, or with `-quiet`, nothing is printed. `NO_COLOR`, `-no_color`, `-msvc_format`, `TERM=dumb` or `CI` give plain lines. `JAIC_PROGRESS=off|plain|live` forces a mode. Diagnostics and program output erase the status line first. The code is `crates/jaic/src/progress.rs`.

`-msvc_format` prints one `path(line,col): error: message` per diagnostic for IDEs. `-timings` prints `jaic-timing: <phase> <seconds> <calls>` per phase, plus `jaic-alloc:` lines for Rust allocations.

### Exit status

| Status | Meaning |
| --- | --- |
| 0 | success |
| 1 | the program didn't compile, a file couldn't be read, or a runtime error stopped it |
| 2 | a mistake on the command line |
| 3 | a build found no `main` |
| 101 | an internal compiler error (please report it) |
| 120 | `JAIC_MEMORY_LIMIT` reached |
| 121 | native code crashed under the interpreter |

With `-interp`, a program that runs to the end exits with its own status. See [diagnostics](../compiler/diagnostics.md#exit-status).

## Configuration

Environment variables (`JAIC_MEMORY_LIMIT`, `JAIC_DIAGNOSTICS`, `JAIC_PROGRESS`, `JACKET_PROJECT`, `NO_COLOR`, ...) are in the [environment variable reference](../reference/environment-variables.md).

## How to change it

Options live in one table, `OPTIONS` in `crates/jaic-cli/src/main.rs`, in `-help` order. It also drives "did you mean" suggestions, `-name=value` splitting and the help text. To add one:

1. Add a field to `Cli` if it stores something.
2. Add a row:

   ```rust
   CliOption::new(&["-no_split"], Action::Switch(|cli| &mut cli.no_split))
       .help(Output, "-no_split", "generate code as one module instead of several"),
   ```

   A value option uses `store!("a directory", |cli, dir| ...)`. A row without `.help(...)` is accepted but not listed. `.aliases(&[...])` adds spellings that are never suggested.
3. Checks involving several options go in `Cli::finish`.

Renamed options go in `RENAMED` (the "use X instead" errors). Tests are in `crates/jaic-cli/tests/` (`cli.rs`, `diagnostics.rs`, `native.rs`).

## Dependencies

[Metaprogram plugins](../metaprogramming/workspaces.md), [build options](../metaprogramming/build-options.md), and the [diagnostics](../compiler/diagnostics.md) renderer.
