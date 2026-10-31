# Diagnostics: errors, warnings and runtime failures

Every message `jaic`, `jailint`, `jaifmt` and `jailsp` show: compile errors, warnings, lints, runtime check failures and command-line mistakes. This page is the style guide and the map of how they are produced.

```
bounds.jai:4:5: error[runtime-error]: runtime error: array bounds check failed: index 3 is outside an array of 3 elements
note: call stack (innermost first):
    `get` at bounds.jai:4
    `main` at bounds.jai:9
help: valid indices are 0 up to the array's count minus one; check the index or the array's length first
```

## Style guide

Say what is wrong in the user's terms, point at the user's own code (the wrong argument, not the whole call), and add a `help:` with a concrete fix, or a `note:` with the why.

- Layout: `error[code]:` / `warning[code]:`, location, snippet, `note:`, `help:`. Only the headline carries the code.
- Start in lowercase, no trailing period. Extra sentences go in separate `note:`/`help:` lines.
- Quote names, types and code with backticks: ``unknown identifier `countr` ``.
- A help that is a code change is a `Suggestion` (`with_suggestion`, `with_replacement`); advice is `with_help`.
- Prefer things the user can type (`-import_dir ../libs`, `#import "Basic";`). Keep internal names out (`__` procedures, `name#N` suffixes).

## How it works

- One type, `Diagnostic` (`jaic-syntax`), serves every tool: a `code`, severity, span, message, label, notes, help lines and suggestions. Severity comes from the code.
- Every code is defined once in `crates/jaic-syntax/src/diagnostics/catalog.rs` with a summary, an optional `explain` and `example`. `Code::TypeMismatch` in Rust is `error[type-mismatch]` for users, `-A`/`-W`/`-D` and `jacket explain`.
- Lint rules are codes too (`Code::Lint(rule)`). The leniency warnings ([leniencies](../language/leniencies.md)) have fixed names.
- `-A <code>` hides a warning, `-W` warns (default), `-D` makes it an error; `-D warnings` and `-A leniencies` work on groups; the last flag wins.
- `jacket explain <code>` prints a code's text; with no argument it lists all codes.
- A `Suggestion` is one fix as edits, read by the CLI, quick fixes and the playground alike.
- Rendering (`render.rs`, one renderer for all tools): `plain` when not a terminal (stable; tests match it), `ascii`, `unicode`, and `msvc` for `-msvc_format`. `jailsp` and the playground always use plain.
- Warnings print before any error and do not change the exit status unless denied. Format strings are checked against argument counts for print-like procedures.
- Runtime failures: the interpreter returns a `Trap` with the unwound frames, and `trap_diagnostic` makes the innermost user frame the location. Wording lives in `ir::check_message`, keyed by `ir::CheckReason`, and executables print the same text.
- A fault inside foreign code run by `-interp` or `#run` is caught by a signal handler (`interp/crash.rs`) that prints the call and exits 121. Wild-pointer loads in compile-time code are caught by `interp/guard.rs` (macOS arm64, Linux x86-64/arm64).
- Command-line mistakes are diagnostics with the codes `cli-usage` and `cli-failure`, with closest-match suggestions.
- `Types::name` shortens long string arguments of polymorphic structs; `Types::full_name` keeps them.

### Exit status

`jaic`: 0 success (`-interp`: the program's own status), 1 compile, runtime, build or link error, 2 command-line mistake, 3 no `main`, 120 memory limit (`JAIC_MEMORY_LIMIT`), 121 native crash under the interpreter, 101 internal error. `jailint`: 1 for a `deny` finding, 2 for a command-line or `jacket.toml` mistake. `jaifmt`: 1 when `--check` finds files that would change.

## How to change it

- New diagnostic: add a variant in the right catalog area (name the problem, not the message), build it at the narrowest span the user wrote, add help or notes. A test must name the code.
- Code never matches message text: sema and editors react to `Diagnostic::code` and `data` (`Data::Scope`, `Data::Module`). Rewording is free, but update tests that match text (`tests/corpus/manifest.json`, `parser/tests.rs`, CLI and language-server tests).
- Renaming or splitting a code breaks `-A`/`-D` users: move the old name to `RETIRED`.
- New runtime check: an `ir::CheckReason` variant with wording in `ir::check_message` (and `runtime_support_check_failed`), raised with `Interp::check_trap`. Helps key off `Trap::kind`.
- The plain layout must stay byte-for-byte stable (`tests/lint/*/bad.snap`). Tools call `render::set_style(detect(choice))` once.

## Configuration

- `--color auto|always|never`; `NO_COLOR`; `CLICOLOR_FORCE`/`FORCE_COLOR`; `TERM=dumb` (plain, no colour).
- `JAIC_DIAGNOSTICS=plain|ascii|unicode` forces a layout; a UTF-8 locale or `WT_SESSION` picks `unicode` on a terminal.

## Dependencies

`diagnostics.rs`, `diagnostics/catalog.rs`, `render.rs`, `suggest.rs`, `sema/suggestions.rs`, `sema/trap_report.rs`, `interp::Trap`, `stdlib/Runtime_Support/module.jai`.
