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

Options go before or after the file. `jaic -help` lists every option; the full table is the [command-line reference](../reference/tools.md#compiling). [`jacket`](jacket.md) runs `jaic` with your arguments plus the project's entry file and settings. Other source files come in through `#load` and `#import`.

## Options

Every option is in the [reference](../reference/tools.md#compiling). Behaviour that the table doesn't show:

- `-check` and `-interp` can't be combined. Build-only options (`-exe`, `-o`, `-sanitize`, ...) are accepted and ignored with either, so one command line can build, check or interpret.
- `-deps` and `-deps_file` list the source files a compile read. CI uses `-deps` to pick jobs ([CI job selection](ci-job-selection.md)); `jacket test` records the lists its own compiles read with `-deps_file` ([jacket test](jacket-test.md#dependency-store)). `-deps_file` writes the file whole, or not at all, and a failed compile writes none.
- `-profile`, `-profile_ops` and `-coverage` (with `-interp`) count what the interpreter runs ([coverage](coverage.md)).
- `-dev_*` flags study the compiler itself and may change at any time.
- A build flavor (`-od`, `-o`, `-very_debug`) overrides a build metaprogram's optimization level; don't combine it with `-O0` to `-O3`.

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

On a terminal `jaic` prints a `Compiling` line, a live status line (`Parsing`, `Checking`, `Running #run`, `Generating`, `Optimizing`, `Linking`) that appears after 150 ms, and a `Finished` line. When stderr is not a terminal, or with `-quiet`, nothing is printed. `NO_COLOR`, `-no_color`, `-msvc_format`, `TERM=dumb` or `CI` give plain lines, which don't redraw. `JAIC_PROGRESS=off|plain|live` forces a mode. Plain lines are coloured when diagnostics would be: on a terminal under `CI`, or anywhere with `FORCE_COLOR` or `CLICOLOR_FORCE` set, unless `NO_COLOR` or `-no_color`. The rule is `color_with` in `crates/jaic-syntax/src/render.rs`; `jacket toolchain add` uses it too. Diagnostics and program output erase the status line first. The code is `crates/jaic/src/progress.rs`.

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

## How to change it

Options are the fields of `Cli` (`crates/jaic-cli-args/src/cli.rs`, and `Tuning` in `tuning.rs`), declared with [`argdef`](argdef.md). The doc comment on a field is its `-help` line, so the struct drives parsing, "did you mean" suggestions, `-name=value` splitting, help and shell completions. To add one:

1. Add a field with an attribute; the section it is listed under is the nearest `section = ...` above it.

   ```rust
   /// generate code as one module instead of several
   #[arg(name = "-no_split")]
   pub no_split: bool,
   ```

   A value option has a type that is not `bool` (`Option<PathBuf>`, `Vec<String>`) and says what it needs: `#[arg(name = "-exe", value = "name", needs = "an executable name")]`. A field without a doc comment is accepted but not listed; `hidden` does the same on purpose. `alias = "..."` adds spellings that are never suggested.
2. Checks involving several options go in `Cli::finish`.

Tests are in `crates/jaic-cli/tests/` (`cli.rs`, `diagnostics.rs`, `native.rs`).

## Configuration and dependencies

Environment variables are in the [reference](../reference/tools.md#environment-variables), and completion scripts in its [shell completions](../reference/tools.md#shell-completions) section. Depends on [metaprogram plugins](../metaprogramming/workspaces.md), [build options](../metaprogramming/build-options.md) and the [diagnostics](../compiler/diagnostics.md) renderer.
