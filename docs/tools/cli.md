# The `jaic` command line

## What it is

`jaic` is the compiler. It has one command and no subcommands. You give it a `.jai` file, and by default it builds a native executable next to that file.

```sh
jaic main.jai                  # build ./main (main.exe on Windows)
jaic main.jai -check           # only type-check; write nothing
jaic main.jai -interp          # run main in the interpreter, no executable
jaic main.jai -interp -- a b   # ...passing arguments to the program
```

Options can go before or after the file. `jaic -help` prints the full list for the binary you have. [`jacket`](jacket.md) runs it with exactly the arguments you give, adding the project's entry file and settings.

The file you name is the only source file on the command line. Other files come in through `#load` and `#import`, as usual.

## How it works

### Choosing what jaic does

| You want to | Run |
| --- | --- |
| build an executable | `jaic main.jai` |
| check that it compiles | `jaic main.jai -check` |
| run it without building | `jaic main.jai -interp` |
| build optimized | `jaic main.jai -o` |
| build for another platform | `jaic main.jai -os windows -cpu arm64` |
| name the output | `jaic main.jai -exe build/app` |

`-check` and `-interp` can't be combined. Options that only make sense for a build (`-exe`, `-o`, `-sanitize`, ...) are an error with either of them.

### Build flavors

| Option | What you get |
| --- | --- |
| *(none)* | unoptimized, with stack traces and runtime checks |
| `-od`, `-optimized_debug` | lightly optimized (LLVM O1), checks kept |
| `-o`, `-optimized` | fully optimized (LLVM O2), no stack traces or runtime checks |
| `-very_debug` | unoptimized, with every runtime check on |
| `-O0` to `-O3` | pick the LLVM level yourself. Don't combine these with the flavors above |

A few switches turn off one thing each: `-no_inline`, `-no_split` (generate one code module instead of several), `-no_debug_info` and `-no_backtrace_on_crash`. `-sanitize address,undefined,fuzzer` builds with the sanitizers (`fuzzer` adds libFuzzer coverage instrumentation), and `-emit_ir out.ll` also writes the LLVM IR.

`-codegen_units N` generates exactly N code modules (`-no_split` is `1`). `-threads N` sizes the one thread pool that code generation and the other parallel work share (default: one per core). `-parse_threads N` sets how many threads parse files ahead of the checker (`0` and `-serial_parse` parse on the checking thread). `-profile` and `-profile_ops` count what the interpreter runs per procedure; `-coverage_file FILE` appends the procedures that ran under `-interp` to FILE. The `-dev_*` flags (`-dev_verify_ir`, `-dev_split_units`, `-dev_no_sink`, `-dev_no_flow`, `-dev_dump_ops`, `-dev_body_probe`, `-dev_body_fifo`, `-dev_o0_passes`) study the compiler itself and may change at any time. None of these are environment variables; `jacket` can set the first three for a project ([jacket](jacket.md)).

If your build metaprogram sets an optimization level, these options override it. `-no_inline` and `-no_split` apply to every workspace jaic writes.

### Where the executable goes

By default the executable is named after the source file and written next to it, unless your build options say otherwise. On Windows it gets `.exe`.

- `-exe app` keeps that location and changes the name.
- `-exe build/app` is relative to the directory you ran jaic from.
- `-output_path out/` sets the directory, and a relative `-exe` goes inside it.

These only affect the main workspace, not other workspaces a metaprogram creates.

### Adding code from the command line

`-add "code"` adds code to the program, and `-run "code"` runs code at compile time, like a `#run`. Both can be repeated. `-I dir` (or `-import_dir dir`) adds a folder to search for modules.

Less common options:

- `-target <triple>` builds for an LLVM target triple, for targets that `-os` and `-cpu` don't cover.
- `-no_dce` also type-checks module code nothing references.
- `-no_check_bindings` sets `CHECK_BINDINGS=false` on a `+Check` plugin.
- `-no_workspace_output` (with `-interp`) doesn't write the files a metaprogram's workspaces ask for.

### Arguments for plugins, metaprograms and programs

A command line can carry arguments for three other parties. jaic splits them like this:

```sh
jaic build.jai +Check - release -- arg1 arg2
#              ^^^^^^ ^^^^^^^^^ ^^^^^^^^^^^^
#              plugin   meta-     program
#                      program   (with -interp)
```

- `+Name` loads the metaprogram plugin `Name`. Everything after it, up to the next `+Name`, `-` or `--`, is that plugin's own options. Module parameters go in the name: `'+Check(CHECK_BINDINGS=false)'`, quoted for the shell. Plugins can't be combined with `-interp`.
- A lone `-` starts the metaprogram's arguments. The metaprogram reads them with `compiler_get_command_line`.
- `--` starts the program's arguments. Only `-interp` runs the program, so `--` with a build or `-check` is an error. The program's `argv[0]` is the source file.

### Output and diagnostics

| Option | Effect |
| --- | --- |
| `-quiet` | print nothing but errors |
| `-verbose` | say what is being done |
| `-no_color` | plain diagnostics. `NO_COLOR` does the same; `FORCE_COLOR` and `CLICOLOR_FORCE` force colour on |
| `-msvc_format` | one `path(line,col): error: message` line per diagnostic, for IDEs that parse that format |
| `-timings` | print each phase's time on stderr as `jaic-timing: <phase> <seconds> <calls>`: `front end`, `workspaces`, `run`, `prepare output`, `codegen`, `link`, `debug info`, `total`, and `interp`, the time compile-time code (`#run`, metaprograms, macros) spent executing, which overlaps `front end` |

### Exit status

| Status | Meaning |
| --- | --- |
| 0 | success |
| 1 | the program didn't compile, a file couldn't be read, or a runtime error stopped it |
| 2 | a mistake on the command line |
| 3 | a build found no `main` to make an executable from |
| 101 | an internal compiler error (please report it) |
| 120 | the `JAIC_MEMORY_LIMIT` was reached |
| 121 | native code crashed under the interpreter |

With `-interp`, a program that runs to the end exits with its own status. [Diagnostics](../compiler/diagnostics.md#exit-status) has more on how errors are reported.

### Mistakes

An unknown option suggests the closest known one. The spellings from before 0.17 (`jaic run x.jai`, `check`, `build`, `--emit-ir`, `--no-debug-info`, `--timings`, `--color`, `--sanitize`, `--target`, `-plug`, `-plugin`, and `-o <path>`) are errors that tell you the new spelling. After a `+Plugin` they are passed to the plugin instead.

## Configuration

| Variable | Effect |
| --- | --- |
| `JAIC_MEMORY_LIMIT` | stop with status 120 once this much memory is allocated (`512M`, `2G`, ...) |
| `JAIC_DIAGNOSTICS` | diagnostic layout: `plain`, `ascii` or `unicode` (default: chosen by terminal) |
| `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE` | turn colour off or on |

## How to change it

Options live in one table, `OPTIONS` in `crates/jaic-cli/src/main.rs`, in `-help` order. Each `CliOption` gives the option's spellings, what it does (`Action`), whether only a build takes it, and its `-help` line. `parse` looks every argument up there, so the table also drives "did you mean" suggestions (over `names`, not `aliases`), `-name=value` splitting, the build-only error and the help text. To add an option:

1. Add a field to `Cli` if it stores something.
2. Add a row to `OPTIONS`:

   ```rust
   CliOption::new(&["-no_split"], Action::Switch(|cli| &mut cli.no_split))
       .build_only()
       .help(Output, "-no_split", "generate code as one module instead of several"),
   ```

   A value option uses `store!("a directory", |cli, dir| cli.output_path = Some(PathBuf::from(dir)))`, or a `store_*` function when the value needs checking. A row without `.help(...)` is accepted but not listed (its spelling is covered by another row's line). `.aliases(&[...])` adds spellings that are never suggested; keep names under three characters there, since `suggest::closest` only matches those exactly.
3. Checks that involve several options or the mode go in `Cli::finish`.

Renamed options go in `RENAMED`, which produces the "use X instead" errors. The `-msvc_format` layout is `Layout::Msvc` in `crates/jaic/src/render.rs`.

Tests live in `crates/jaic-cli/tests/`: `cli.rs` for parsing, `diagnostics.rs` for output and exit codes, and `native.rs` for builds.

## Dependencies

- [Metaprogram plugins](../metaprogramming/workspaces.md): what `+Name` loads.
- [Build options](../metaprogramming/build-options.md): what the flavor and output options set.
- [Diagnostics](../compiler/diagnostics.md): the renderer behind `-no_color`, `-msvc_format` and `JAIC_DIAGNOSTICS`.
