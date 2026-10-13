# The `jaic` command line

## What it is

One command, no subcommands:

```
jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]
```

By default `jaic` builds a native executable. `-check` only compiles the program and writes nothing; `-interp` runs `main` in the interpreter. Options may come before or after the file. The file is the one argument that starts with neither `-` nor `+`; a second one is an error (`#load` or `#import` the others from the first). `-check` and `-interp` together are an error.

Run `jaic -help` for the list the binary itself prints.

## Grammar

- `+Name` starts a plugin ([metaprogram plugins](../metaprogramming/metaprogram-plugins.md)). Every argument after it, up to the next `+Name`, a lone `-` or `--`, belongs to that plugin alone. Put the file and jaic's own options before the first plugin. Module parameters are part of the name, `'+Check(CHECK_BINDINGS=false)'`; quote it, since shells read parentheses. Plugins do not combine with `-interp`.
- A lone `-`: everything after it (up to `--` with `-interp`) is for the metaprogram, read with `compiler_get_command_line`.
- `--`: everything after it is the program's own arguments. Only `-interp` runs the program, so `--` with a build or `-check` is an error. `argv[0]` is the source file.

## Options

| Option | Meaning |
|---|---|
| `-check` | compile only; run `#run` code, write nothing |
| `-interp` | run `main` in the interpreter |
| `-add "code"` | add the code to the target program (repeatable, in order) |
| `-run "code"` | run the code at compile time, as a `#run` (repeatable) |
| `-import_dir dir`, `-I dir` | also look for modules in `dir` (repeatable) |
| `-os os`, `-cpu cpu`, `-target triple` | the target: `linux`, `windows`, `macos`, `wasm`; `x64`, `arm64`; an LLVM triple |
| `-no_dce` | type-check unreferenced module code too |
| `-no_check` | accepted and ignored: the `Check` plugin runs only when asked for with `+Check` |
| `-no_check_bindings` | `CHECK_BINDINGS=false` for a `+Check` that has no value for it; otherwise nothing |
| `-no_backtrace_on_crash` | no crash handler in the target program |
| `-no_color` | no colour in diagnostics (`NO_COLOR`, `FORCE_COLOR` and `CLICOLOR_FORCE` work as before) |
| `-msvc_format` | one `path(line,col): error: message` line per diagnostic, notes as `path(line,col): note: ...` |
| `-quiet` | print nothing but errors |
| `-verbose` | say what is being done: the file, the plugins, the workspaces a metaprogram built, the output written |
| `-timings` | wall time of each compiler phase on stderr, as `jaic-timing: <phase> <seconds> <calls>` lines |
| `-help`, `-h`, `--help` | the help text |
| `-version`, `-V`, `--version` | the version |

Building only (an error with `-check` or `-interp`):

| Option | Meaning |
|---|---|
| `-exe name` | the target workspace's `output_executable_name`. It may carry a directory and an extension: `-exe build/app.exe` |
| `-output_path dir` | the target workspace's `output_path` |
| `-o`, `-optimized` | optimized build: LLVM O2 and no stack trace or runtime checks, as `set_optimization(*o, .OPTIMIZED)` plus `stack_trace = false` |
| `-od`, `-optimized_debug` | lightly optimized (LLVM O1) with stack traces and runtime checks kept |
| `-very_debug` | unoptimized, every runtime check on (`.VERY_DEBUG`) |
| `-O0` .. `-O3` | pick the LLVM level directly; not combined with `-o`, `-od` or `-very_debug` |
| `-no_inline` | no inlining of procedures |
| `-no_split` | one codegen unit, as `JAIC_CODEGEN_UNITS=1` |
| `-emit_ir file.ll` | also write the LLVM IR |
| `-no_debug_info` | leave out native debug information |
| `-sanitize list` | `address` and/or `undefined`, comma-separated |

`-interp` only: `-no_workspace_output` does not write what a metaprogram's workspaces ask for (sweeps over many programs).

The `-o`, `-od`, `-very_debug` and `-O` options replace what the metaprogram chose for the optimization level, as `-O` always did. `-no_inline` and `-no_split` hold for every workspace jaic writes.

### Where the executable goes

Without `-exe` and `-output_path` the executable is named after the source file and written next to it, unless the metaprogram's build options say otherwise. `-exe name` with a bare name keeps that place; with a directory part (`-exe build/app`, `-exe /tmp/app`) the path is taken from the directory jaic was started in. `-output_path dir` (also from where jaic was started) holds the executable, and a relative `-exe` goes below it. A name without an extension gets the platform's (`.exe` on Windows). The target program's `get_build_options` reports these values, so a metaprogram that passes the options on keeps them. They do not reach other workspaces a metaprogram creates; those use their own `output_path` and `output_executable_name`.

## Errors

- `jaic run x.jai`, `jaic check x.jai` and `jaic build x.jai` (when no file of that name exists) end in "jaic has no subcommands since 0.17.0" and the same command in the new form, other arguments carried over and renamed.
- The old spellings `--emit-ir`, `--no-debug-info`, `--timings`, `--color`, `-color`, `--sanitize`, `--target`, `-plug` and `-plugin` are errors that name the new spelling. After a `+Plugin` they are the plugin's own arguments and are not read.
- `-o` followed by a word that is neither an option nor a `.jai` file is an error: `-o` takes no value since 0.17.0.
- An unknown option suggests the nearest known one.
- Build-only options with `-check` or `-interp`, `-no_workspace_output` without `-interp`, and `-quiet` with `-verbose` are command-line errors.
- Exit status 2 is a command-line mistake. The other statuses are in [diagnostics](../compiler/diagnostics.md#exit-status).

## Coming from 0.16

| 0.16 | Now |
|---|---|
| `jaic run file.jai` | `jaic file.jai -interp` |
| `jaic run file.jai -- args` | `jaic file.jai -interp -- args` |
| `jaic check file.jai` | `jaic file.jai -check` |
| `jaic build file.jai` | `jaic file.jai` |
| `jaic build file.jai -o out` | `jaic file.jai -exe out` (next to the file) or `jaic file.jai -exe build/out` (from the current directory) |
| `jaic build file.jai -o build/app.exe` | `jaic file.jai -exe build/app.exe`, or `-output_path build -exe app.exe` |
| `-O2` | `-O2`, or `-o` for the optimized flavor |
| `--emit-ir file.ll` | `-emit_ir file.ll` |
| `--no-debug-info` | `-no_debug_info` |
| `--timings` | `-timings` |
| `--sanitize list`, `--target triple` | `-sanitize list`, `-target triple` |
| `--color never` | `-no_color` |
| `--color always` | `FORCE_COLOR=1` |
| `-plug Name`, `-plugin Name` | `+Name` after the file and jaic's options |
| `-plug Name -x y` (options shared by all plugins) | `+Name -x y` (options of that plugin) |
| `jaic help`, `jaic version` | `jaic -help`, `jaic -version` |

## How to change it

The parser is `parse` in `crates/jaic-cli/src/main.rs`; `KNOWN_OPTIONS`, `VALUE_OPTIONS`, `BUILD_ONLY` and `RENAMED` list the options for suggestions and mode errors, and `usage_text` is the help. A new option goes in the `match` of `parse`, in those lists and in the help text. The `-msvc_format` layout is `Layout::Msvc` in `crates/jaic/src/render.rs`.

Tests: `crates/jaic-cli/tests/diagnostics.rs` (`command_line_mistakes_explain_themselves`), `cli.rs` (plugins, `-add`/`-run`, `-timings`, chatter levels), `native.rs` (`exe_and_output_path_place_the_executable`, `optimization_flavors_build_and_run`).

## Dependencies

[Metaprogram plugins](../metaprogramming/metaprogram-plugins.md), [build options](../metaprogramming/build-options.md), [diagnostics](../compiler/diagnostics.md).
