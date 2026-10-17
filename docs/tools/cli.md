# The `jaic` command line

## What it is

One command, no subcommands:

```
jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]
```

By default it builds a native executable. `-check` compiles and writes nothing; `-interp` runs `main` in the interpreter (the two together are an error). The file is the one argument starting with neither `-` nor `+`; load others with `#load`/`#import`. `jaic -help` prints the binary's own option list.

```sh
jaic main.jai                    # build next to the source
jaic main.jai -check             # compile only
jaic main.jai -interp -- a b     # run with program arguments
jaic main.jai -O2 -exe build/app # optimized, placed below the current directory
jaic build.jai +Check - release  # plugin, then metaprogram arguments
```

## Grammar

- `+Name` starts a [plugin](../metaprogramming/workspaces.md); everything after it up to the next `+Name`, a lone `-` or `--` belongs to that plugin. Module parameters are part of the name (`'+Check(CHECK_BINDINGS=false)'`, quoted). Plugins do not combine with `-interp`.
- A lone `-`: the rest (up to `--` with `-interp`) is for the metaprogram, read with `compiler_get_command_line`.
- `--`: the program's own arguments (`argv[0]` is the source file). Only `-interp` runs the program, so `--` otherwise is an error.

## Options

Common: `-check`, `-interp`, `-add "code"` and `-run "code"` (repeatable), `-import_dir dir`/`-I dir`, `-os`/`-cpu`/`-target`, `-no_dce` (type-check unreferenced module code too), `-no_check_bindings` (`CHECK_BINDINGS=false` for `+Check`), `-no_backtrace_on_crash`, `-no_color` (`NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE` also apply), `-msvc_format` (`path(line,col): error: message`), `-quiet`, `-verbose`, `-timings` (`jaic-timing: <phase> <seconds> <calls>` on stderr), `-help`, `-version`.

Build only (an error with `-check`/`-interp`): `-exe name`, `-output_path dir`, `-o`/`-optimized` (LLVM O2, no stack trace or runtime checks), `-od`/`-optimized_debug` (O1, checks kept), `-very_debug`, `-O0`..`-O3` (not combined with the flavors), `-no_inline`, `-no_split` (one codegen unit, like `JAIC_CODEGEN_UNITS=1`), `-emit_ir file.ll`, `-no_debug_info`, `-sanitize address,undefined`. `-interp` only: `-no_workspace_output`. The optimization flags replace what the metaprogram chose; `-no_inline` and `-no_split` hold for every workspace jaic writes.

**Where the executable goes.** By default it is named after the source and written next to it unless build options say otherwise. `-exe name` keeps that place; with a directory part the path is relative to where jaic started. `-output_path dir` holds the executable and a relative `-exe` goes below it. A name without an extension gets the platform's. These do not reach other workspaces a metaprogram creates.

## Errors

Exit status 2 is a command-line mistake (other statuses: [diagnostics](../compiler/diagnostics.md#exit-status)). An unknown option suggests the nearest known one. Pre-0.17 forms (`jaic run x.jai`, `check`, `build`, `--emit-ir`, `--no-debug-info`, `--timings`, `--color`, `--sanitize`, `--target`, `-plug`, `-plugin`, and `-o` with a value) are errors that name the new spelling; after a `+Plugin` they are the plugin's own arguments. Build-only options with `-check`/`-interp`, `-no_workspace_output` without `-interp`, and `-quiet` with `-verbose` are errors.

## How to change it

The parser is `parse` in `crates/jaic-cli/src/main.rs`; `KNOWN_OPTIONS`, `VALUE_OPTIONS`, `BUILD_ONLY` and `RENAMED` drive suggestions and mode errors, and `usage_text` is the help. A new option goes in the `match` of `parse`, those lists and the help. The `-msvc_format` layout is `Layout::Msvc` in `crates/jaic/src/render.rs`. Tests: `crates/jaic-cli/tests/diagnostics.rs`, `cli.rs`, `native.rs`.

## Dependencies

[Metaprogram plugins](../metaprogramming/workspaces.md), [build options](../metaprogramming/build-options.md), [diagnostics](../compiler/diagnostics.md).
