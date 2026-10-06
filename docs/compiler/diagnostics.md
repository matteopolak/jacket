# Diagnostics: errors, warnings and runtime failures

## What it is

Every message `jaic`, `jailint`, `jaifmt` and `jailsp` show the user: compile errors, lints, runtime check failures (under `jaic run`, in `#run` code, and in built executables) and command-line mistakes. This page is the style guide for writing them, and it explains how they are rendered.

## Style guide

**Rule 1: say what the problem is, and how to get what the user wants.** Each diagnostic:

- states the problem plainly, in the user's terms rather than the compiler's (`` cannot declare `r` from `random_seed(...)`, which has no value ``, not `expression has no value`);
- points at the user's own code: the argument that is wrong rather than the whole call, the user's call into the standard library rather than the line deep inside it;
- adds a `help:` line with a concrete fix when one is clear (``help: convert with `cast(s64)`, which drops the fraction``), or a `note:` that explains why, when there is no single fix (``note: this compile targets `OS == .MACOS` ...``).

A message the user cannot act on is a bug in the message.

The other rules:

- rustc's layout: `error:` / `warning:`, the location, the snippet, then `note:` and `help:` lines.
- Messages start in lowercase and have no trailing period. Several sentences go in separate `note:`/`help:` lines.
- Names, types, tokens and code are quoted with backticks: `` unknown identifier `countr` ``, `` expected `;` after statement, found `}` ``. Not single quotes.
- Types are shown as jaic names them (`s64`, `float32`); the declared alias (`int`) appears in the snippet.
- A help that is a code change uses `with_fix` (shown as a diff outside the plain layout); one that is advice uses `with_help`.
- Prefer naming things the user can type: `jaic check file.jai`, `-import_dir ../libs`, `#import "Basic";`.
- Paths are shown relative to where the tool was started when they are inside it (`jaic::display_path`).
- Internal names stay out: `__`-prefixed procedures, `name#N` polymorphic suffixes, the interpreter's thunks (`#run`/`#const` code is described as such).

## How it works

### The `Diagnostic` value (compiler)

`source::Diagnostic` holds the severity, the primary `span`, the `message`, an optional `label` (text under the carets), `notes` (each with a span, or `Span::NONE` for text only), `help` lines and an optional `fix` (span + replacement). Builders: `Diagnostic::error(span, msg).with_label(..).with_note(span, ..).with_help(..).with_fix(help, span, text)`. Sema returns `Err(Box<Diagnostic>)`; `err(span, msg)` is the short form for a message with nothing else.

Suggestions that need extra work are computed when the error is rendered, not when it is created, because failed lookups are routine while checking overloads and `#if`s (`sema/suggestions.rs`):

- unknown identifier: a visible name within edit distance (`suggest::closest`); else a build metaprogram next to the file that adds the name with `add_build_string`; else the stdlib module that declares it (`` help: `print` is declared in the `Basic` module: add `#import "Basic";` ``).
- missing module (`sema/modules.rs`): the directories searched, a module with a close name, or a folder nearby that holds it and the `-import_dir` that finds it.
- missing `#load` file: where it looked and a file with a close name.
- unknown member: the closest member, or the list of members.
- call mismatches (`sema/calls.rs`, `call_mismatch`): the error moves to the argument concerned and a note shows the procedure's declaration; a misspelled named argument gets the closest parameter.
- type mismatches (`sema/convert.rs`, `conversion_help`): the usual conversions (`cast`, `.data`/`to_c_string`, `tprint`, `.*`, `*value`); a declaration's mismatch points at the value with the declared type as a note, and a `return`'s at the returned value with the procedure's return type as a note (`return_type_mismatch` in `sema/stmt.rs`).

### Rendering (`crates/jaic/src/render.rs`)

One renderer serves `jaic`, `jailint` and `jaifmt`'s command-line errors (`jaifmt` is a Jai program and prints the plain form itself). `Report` is the renderer's input: severity, optional code (a lint's rule), message, a primary `Label`, secondary labels, notes and helps (a help may carry a `Fix`, shown as changed lines). `Diagnostic::report` converts a compiler diagnostic; `Report::from_text` turns text with `help:`/`note:` lines (errors from the backend, the linker or a config parser) into one.

Three layouts, picked once per process (`render::detect`, then `set_style`):

| Layout | When | Looks like |
|---|---|---|
| `plain` | stderr is not a terminal, or `TERM=dumb` | `path:line:col: error: message`, the line, carets, then `note:`/`help:` lines. jailint keeps its `-->` block. Stable: tests and tools match it. |
| `ascii` | a terminal without a UTF-8 locale | rustc-like block: gutter line numbers, a context line either side, labels for every span, multi-line spans with a connector, fixes as `~` lines |
| `unicode` | a terminal with a UTF-8 locale, or Windows Terminal | the same with box drawing (`╭─[`, `│`, `━`) |

Example (`unicode`):

```
error: type mismatch: expected `s64`, found `float32`
   ╭─[t10.jai:1:39]
 1 │ main :: () { a: float = 1.0; b: int = a; }
   ·                                       ━
   ·                                 ─── expected because of this type
   ╰─
   = help: convert with `cast(s64)`, which drops the fraction
```

The same error in the `plain` layout:

```
t10.jai:1:39: error: type mismatch: expected `s64`, found `float32`
    main :: () { a: float = 1.0; b: int = a; }
                                          ^
t10.jai:1:33: note: expected because of this type
    main :: () { a: float = 1.0; b: int = a; }
                                    ^^^
help: convert with `cast(s64)`, which drops the fraction
```

Colour is separate from the layout: errors red, warnings yellow, notes cyan, helps green, the gutter blue, primary carets in the severity's colour and secondary labels blue. Long lines are cut around their labels and tabs are expanded outside the plain layout. `jailsp` and the browser playground always use plain text.

### Runtime failures (`sema/trap_report.rs`)

When interpreted code fails a check (array bounds, a narrowing cast whose value does not fit, a `#complete` switch that matches no case, null pointer, stack overflow, division by zero, a procedure that falls off its end without returning a value, `#asm` faults, an assertion), the interpreter returns a `Trap` with the message, the statement, and the procedures it unwound through (`Trap::frames`, filled in `Interp::exec`, capped at 64). `Compiler::trap_diagnostic` turns it into a diagnostic:

- the primary location is the innermost frame in the user's own code; when the check failed inside the standard library, a note says which procedure it failed in;
- `note: call stack (innermost first):` lists the frames, with library frames kept, internal names left out and recursion folded;
- a `help:` explains the common checks (valid index range, the range a cast's target holds and how to truncate on purpose, a `#complete` switch's default label, null pointers, recursion depth, missing `return`, a zero divisor);
- for `#run` code, a note marks the directive that started it.

```
bounds.jai:4:5: error: runtime error: array bounds check failed: index 3 is outside an array of 3 elements
        return xs[i];
        ^^^^^^^^^^^^
note: call stack (innermost first):
    `get` at bounds.jai:4
    `main` at bounds.jai:9
help: valid indices are 0 up to the array's count minus one; check the index or the array's length first
```

A failed `assert` goes through `runtime_support_report_assertion` (`stdlib/Runtime_Support.jai`). Under the interpreter it is a `#compiler` hook (`Hook::AssertionFailed`) that raises a trap carrying the assert's location (`Trap::assertion`), so the report is `error: runtime error: assertion failed: <message>` at the `assert` line (or `` `cond` is false `` when there is no message), with the assertion machinery left out of the stack. In a built executable the same procedure's body prints `path:line:col: error: assertion failed: <message>` and `call stack (innermost first):` from `context.stack_trace`. Paths under the directory the build started in are shown relative to it, like the interpreter's: jaic provides that directory as `__jaic_build_directory: string #elsewhere` (`build_directory_global` in `sema/runtime_info.rs`, from `jaic::display_base`), and `runtime_support_shown_path` strips it in the stack trace, `write_loc` and `write_failure_location`.

The checks' messages come from one place, `ir::check_message`, keyed by the `ir::TRAP_*` reason. In a built executable a failed check calls `runtime_support_check_failed(reason, a, b, fatal, line, filename)` in `stdlib/Runtime_Support.jai` (resolved into `ir::Program::check_failed` by `note_check_handler` in `sema/expr.rs`), which prints the same message as `path:line: error: <message>` with the path relative to the build's directory, and the program then traps. A `.NONFATAL` check prints `warning:` instead and goes on, in both backends.

A metaprogram's own error (`compiler_report`, `compiler_set_workspace_status(.FAILED)`) sets `Trap::reported`: it is shown as the metaprogram wrote it, at the place it named, without the compile-time-execution prefix.

### Command-line errors

`jaic`'s argument parser returns a `CliError` (message + helps): unknown commands and options get the closest match, misplaced files and options say where they go, `-os`/`-cpu` list their values. Input files are checked before compiling (missing file with a close name, a directory with its entry file, permissions). `jailint`, `jaifmt` and `jailsp` follow the same rules; `jailsp` run by hand says it is a language server and how to check a file instead.

Build problems are caught before the backend runs where possible: an output path that is a directory or cannot be written, a program without `main`, an unknown sanitizer. A linker that is missing says how to install one (and that `jaic run` needs none); a failed link keeps the linker's output and adds a help when it says a library or symbol is missing.

A Rust panic is reported as `error: internal compiler error: ...` with a note and the issue tracker link (`install_panic_hook`).

### Exit status

| Tool | 0 | 1 | 2 | other |
|---|---|---|---|---|
| `jaic` | success (`run`: the program's own status) | compile error, runtime error, build or link failure, unreadable input | command-line mistake | 120 memory limit (`JAIC_MEMORY_LIMIT`), 101 internal compiler error |
| `jailint` | no `deny` findings | a `deny` finding | command-line or `jailint.toml` mistake, unreadable path | |
| `jaifmt` | success | `--check`: files would change | errors (command line, config, unreadable or unformattable files) | |
| `jailsp` | `exit` after `shutdown` | input ended or the protocol broke | command-line mistake | |

## How to change it

- New compile error: build the `Diagnostic` where the problem is found, at the narrowest span the user wrote. Add a `with_help` when the fix is clear; a `with_note(span, ..)` for a related place (a declaration, the type a value had to match). Add a case to `crates/jaic-cli/tests/diagnostics.rs`.
- Changing an existing message breaks substring checks: `tests/corpus/manifest.json` (`negative` cases; run `python3 tools/jaic-sweep.py negative`), `tests/stdlib-targets.txt` (first errors of the stdlib target check), parser tests in `crates/jaic/src/parser/tests.rs`, `crates/jaic-cli/tests/cli.rs`, the language server's tests and docs that quote messages. `tools/jaic-diff.py` and `tools/jaic-sweep.py` match a few messages too (`has no \`main\` procedure`, `is not available here`).
- New runtime check: give the trap a message in the interpreter (`Interp::trap`), and if it is common, a help in `help_for` (`trap_report.rs`).
- Layout or colour changes: `render.rs`, tests in `render/tests.rs`. The plain layout must stay byte-for-byte stable; `jailint`'s `tests/lint/*/bad.expected` and the jaic tests depend on it.
- A command-line tool that prints errors should call `jaic::render::set_style(detect(choice))` once and print `Report`s, so `--color`, `NO_COLOR` and `JAIC_DIAGNOSTICS` behave the same everywhere.

## Configuration

| Setting | Effect |
|---|---|
| `--color auto\|always\|never` (`jaic`, `jailint`, `jaifmt`) | colour; `auto` (the default) colours terminals |
| `NO_COLOR` (any value) | no colour under `auto` |
| `CLICOLOR_FORCE`, `FORCE_COLOR` (set, not `0`) | colour under `auto` even when not a terminal |
| `TERM=dumb` | plain layout, no colour |
| `JAIC_DIAGNOSTICS=plain\|ascii\|unicode` | force a layout |
| `LC_ALL` / `LC_CTYPE` / `LANG` with `UTF-8`, or `WT_SESSION` | `unicode` layout on a terminal |

On Windows, `auto` colours only consoles known to accept ANSI sequences (Windows Terminal, ConEmu, ANSICON, or a `TERM`).

## Dependencies

No crates beyond the standard library: `render.rs` (layouts, colour), `suggest.rs` (edit distance), `sema/suggestions.rs`, `sema/trap_report.rs`, `interp::Trap`, `stdlib/Runtime_Support.jai` (assertion reports in executables).

## What the upstream corpus shows

`jaic check` (and the curated cases' `run`) over every entry point in `corpus/upstream` (931 invocations, `tools/fetch_upstreams.py` projects) ranked the messages users meet most. Most failures are programs that need their build metaprogram, a module path or an `#import`; the improvements above target those first.

| Count | Message (first error of a failing invocation) | Before | After |
|---:|---|---|---|
| 56 | unknown identifier | ``unknown identifier 'print'``, sometimes a far-fetched similar name | the build metaprogram that adds the name (`add_build_string`), else a similar name, plus the stdlib module that declares it: ``help: `print` is declared in the `Basic` module: add `#import "Basic";` `` |
| 28 | module not found | where it looked, generic help | the folder nearby that holds the module and the `-import_dir` that finds it |
| 11 | type has no member | ``type bool has no member 'open'`` | the closest member as a fix, or the list of members |
| 9 | expected `;` after statement | ``found '}'`` for `f({1})` | help: struct literals in expressions start with a dot (`.{1, 2}`) |
| 8 | type mismatch: expected T, found T | the whole declaration underlined | the value underlined, the declared type as a note, a conversion help (`cast`, `.data`, `tprint`, `.*`) |
| 7 | `#asm`: expected a general-purpose register or memory destination | - | unchanged (a jaic gap, see below) |
| 5 | unknown library | ``unknown library 'x'`` | how to declare it with `#library`, and that `#if OS` declarations exist only for that OS |
| 5 | #assert failed | `#assert failed` | the condition (`` `OS == .WINDOWS` is false ``), the target being compiled for, or what `#assert(false)` means |
| 4 | cannot declare a variable from an expression with no value | as is | ``cannot declare `r` from `random_seed(...)`, which has no value`` + help |
| 4 | in call to: argument type does not match | the whole call underlined | the argument underlined, the declaration as a note, a conversion help |
| 4 | in call to: unknown identifier | reported as a call problem | reported as the argument's unknown identifier, with the suggestions above |
| 3 | in call to: no parameter named | - | the closest parameter, or the parameter list |
| 3 | expression has no value (stdlib `File_Async`) | a stdlib bug | fixed: the module checks |
| 1 | type mismatch: `u8` and `string` | - | ``help: `"0"` is a string; for the character's code write `#char "0"` `` |
| 1 | metaprogram marked the workspace as failed | ``error during compile-time execution: error: The workspace was marked as failed by its metaprogram.`` | ``error: the metaprogram marked the workspace as failed`` at the user's call |

The rest were single occurrences. Crashes and limits (5): two segfaults inside native SDL/GL calls made by a `#run` metaprogram (a program bug, but jaic dies without a message), one timeout (a large project) and two `JAIC_MEMORY_LIMIT` stops (a jaic bug, see below).

Found along the way, not fixed (jaic gaps rather than user errors):

- `#asm`: `pmovmskb.x found_gpr:, results_vec;` (a register declared in the destination) is rejected (`smari--jai-xml`, 7 programs).
- Mixed declare-and-assign whose assigned target is not a plain name, `ok:, toki.str = f();`, does not parse (`sjorsdonkers--toml-jai`, 6 programs).
- Positional struct literals without a dot, `f({1})` and `f({.A, 1})`, do not parse (`jai_parser` tests, `UnNabbo--no_api`); the error now suggests `.{...}`.
- `stdlib/Bindings_Generator`: `Enum.enumerates` holds `Enumerate` values; generators written for it as `*Declaration`s (`ostef--Vk-Engine`) fail to type-check.
- `print("% % %\n", root.kind, expressions.count, code_to_string(compiler_get_code(root)))` in `#run` exceeds 3 GiB; the same calls as separate statements work (`withlang-dev--open-jai`, 2 programs).
- A fault in native code called from `#run` (here SDL/GL without initialisation) kills jaic with SIGSEGV and no message.

