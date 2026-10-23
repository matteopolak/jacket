# Diagnostics: errors, warnings and runtime failures

## What it is

Every message `jaic`, `jailint`, `jaifmt` and `jailsp` show: compile errors, warnings, lints, runtime check failures (under `jaic -interp`, in `#run`, and in built executables) and command-line mistakes. This page is the style guide and explains how they are produced and rendered.

## Style guide

Each diagnostic says what the problem is in the user's terms, points at the user's own code (the wrong argument, not the whole call; the user's call into the stdlib, not a line deep inside it), and adds a `help:` with a concrete fix when one is clear, or a `note:` explaining why when there is none. A message the user cannot act on is a bug.

- rustc's layout: `error:` / `warning:`, location, snippet, then `note:` and `help:`.
- Messages start in lowercase with no trailing period. Several sentences go in separate `note:`/`help:` lines.
- Names, types, tokens and code are quoted with backticks, not single quotes: `` unknown identifier `countr` ``.
- Types are shown as jaic names them (`s64`); a help that is a code change is a `Suggestion` (`with_suggestion`, or `with_replacement` for one replaced span), advice uses `with_help`.
- Prefer things the user can type: `jaic file.jai -check`, `-import_dir ../libs`, `#import "Basic";`.
- Keep internal names out: `__` procedures, `name#N` polymorphic suffixes, interpreter thunks (`#run`/`#const` code is described by `ir::FuncOrigin`).

## How it works

### `Diagnostic` (compiler)

`source::Diagnostic` holds severity, primary `span`, `message`, optional `label`, `notes` (with a span, or `Span::NONE` for text only), `help` lines and `suggestions`.

```rust
Diagnostic::error(span, msg).with_label(..).with_note(span, ..).with_help(..).with_replacement(title, span, text)
```

### Suggestions

A `Suggestion` (`source/suggestion.rs`) is one way to fix the diagnostic, in a form the CLI prints and editors apply. It has a `title` (the `help:` line), `edits`, an `Applicability` and optionally an editor `action` title. The CLI, `jailsp` quick fixes and the playground all read the same list, so a hint is written once.

- `Edit::Replace { span, text }` replaces the text of `span`. An empty span inserts (`Suggestion::insert`), an empty `text` deletes. A suggestion may hold several edits (all in the diagnostic's file), applied together.
- `Edit::Import(ImportSuggestion)` adds an `#import` to the diagnostic's file. The editor picks the place (after the last import, else above the first code) and skips it when the file already has it.
- `Applicability::MachineApplicable` means the result still means what the author wrote; editors mark it preferred. `MaybeIncorrect` is a guess the author should check (a similar name, a cast), never preferred.
- `listed: false` (`Suggestion::unlisted`) keeps the CLI from printing the title, for alternatives an earlier help line names together (the other modules that declare a name).

```rust
use crate::source::{Applicability, Suggestion};

// One replaced span (a `MaybeIncorrect` guess, editor action "Replace with `count`"):
d.with_replacement("a similar name exists: `count`", name_span, "count")

// Several edits, or machine-applicable:
d.with_suggestion(Suggestion::with_edits(
    "write `M.operator +(u, v)`",
    vec![Edit::Replace { span: whole_expr, text: format!("M.operator +({a}, {b})") }],
    Applicability::MaybeIncorrect,
))
// Offer the same fix as an insert, e.g. a missing `;`:
d.with_suggestion(Suggestion::insert("add `;`", end_of_previous, ";", Applicability::MachineApplicable))
```

Gotchas: take edit spans and the text between them from `self.src.sources.snippet(span)`, and parenthesise an operand when the new text changes precedence. A suggestion never goes with a span from another file than the diagnostic's (editors drop it). Order: suggestions print before plain `help:` lines, in the order added. Add several suggestions for alternatives rather than one with a choice in the title.

Sema returns `Err(Box<Diagnostic>)`; `err(span, msg)` is the short form. Extra suggestions are computed at render time, not creation, because failed lookups are routine during overload and `#if` checks (`sema/suggestions.rs`): unknown identifiers (a build metaprogram that adds the name, close names, a namespaced import such as `M :: #import "Math"` that exports it, which gets an insert-`M.` suggestion and a `using M;` one in place of a duplicate `#import` (`sema/namespace_hints.rs`), stdlib modules declaring it, one `Edit::Import` suggestion each, the first listed and machine-applicable when it is the only module), missing modules and `#load` files (searched directories, close names), unknown members (the closest field, `using` member, constant or procedure of a type, the closest export of a module with `UnknownModuleMember`, "is not exported" for a `#scope_file` name, and the other modules that declare it), call mismatches (`call_mismatch` moves the error to the argument and notes the declaration; a `DiagnosticKind::ArgumentCount` error also notes the callee's parameters, and extra arguments get a removal suggestion, `sema/calls/count_hints.rs`), and type mismatches (`conversion_hint` suggests `cast`, `.data`, `tprint`, ...).

Conversion and cast errors rewrite the code through one helper, `sema/convert/wrap.rs`: `wrap_suggestion` puts `cast(T)`, `.*`, `*` or `!= 0` around the expression at a span, parenthesising anything that is not a single primary expression (`cast(s32) (a + 1)`). It is used for assignments and returns (`implicit.rs`), call arguments (`calls/cost.rs`), binary operands (`binary/typing.rs`, which also passes the operand spans so the edit never covers the whole expression) and `s32(x)` calls (`calls/mod.rs`). The same area suggests `for 0..n - 1` for `for n`, drops extra `return` values, removes a `.*` on a non-pointer, and adds `,trunc` or `,force` to a `cast`. All of them are `MaybeIncorrect`: a cast can drop bits and a dereference can read null, so editors offer them but never apply them on their own. A new wrap form goes in `Wrap` and `wrapped_text`; code from a macro has no source text and gets the advice without an edit.

#### Hints at the error

Errors in calls, declarations and assignments carry their hints from creation, since the context is at hand:

- Redeclarations note the first declaration (`stmt/locals.rs`, `scope/bindings.rs`, `structs/layout.rs`, `enum_hints.rs`, `using.rs`). Use the same wording: `` `x` is already declared in this scope `` with the label `declared again here` and the note `` `x` is first declared here ``.
- Assigning to a constant (`stmt/assign_hints.rs`) notes the declaration and replaces its `::` with `:=` (`MaybeIncorrect`) when it is written `NAME :: value;`. An immutable argument gets `s := s;` inserted after the body's `{`; a member reached through `using` only gets the text, since a copy would not change which one the name means. An expression that is not a place says what it is (a literal, a call, an operator).
- A discarded `#must` result offers `_ := f();` (one `_` per result) when the call starts its statement (`check_must_used`).
- Counts: struct literals (`structs/literal_hints.rs`), names against values (`decl_count_hints.rs`) and lambda arity note the expected members or signature, and extra values are removable. An enum value outside its type widens the header type (`enum_hints.rs`; the span is kept in `TypeState::enum_base_spans`).

Removing something is always `MaybeIncorrect`, never preferred.

Lexer and parser errors carry suggestions too, built where they are raised: `Parser::missing_semicolon` (`parser/mod.rs`) inserts `;` where the previous token ends (machine-applicable when the next token starts a line or closes a block, a guess when more of the line follows), `Parser::unclosed_block` adds a "the block opens here" note at the `{` and, at the end of the file, a suggestion that writes the `}`, and the lexer fixes an unknown escape (`\q` to `\\q`) and closes an unterminated block comment or here-string at the end of the file. A parser path that rebuilds an error to move its span (the editor's recovery) must keep the rest with `Diagnostic { span, ..error }`, or the suggestions are lost. `jailsp` keeps them next to the syntax diagnostics (`Analysis::syntax_fixes`) and `suggestion_fixes.rs` turns them into quick fixes like the type checker's.

### Rendering (`crates/jaic/src/render.rs`)

Line and column numbers come from `jaic::source::LineIndex` (see [source positions](source-positions.md)); a `Label` or `Fix` carries the caller's index so rendering stays linear. One renderer serves `jaic`, `jailint` and `jaifmt`'s command-line errors. `Report` is its input; `Diagnostic::report` converts a compiler diagnostic (each listed suggestion is a `help:` line; one whose edits are replacements in one file shows the changed code under it, as `~`, or `-` for removed lines) and `Report::from_text` converts text with `help:`/`note:` lines. Layouts, picked once per process (`render::detect`, then `set_style`):

| Layout | When |
|---|---|
| `plain` | stderr not a terminal, or `TERM=dumb`: `path:line:col: error: message`, line, carets, `note:`/`help:`. Stable: tests and tools match it. |
| `ascii` | terminal without UTF-8 locale |
| `unicode` | terminal with UTF-8 locale, or Windows Terminal (box drawing) |
| `msvc` | `jaic -msvc_format`: `path(line,col): error: message`, no snippet or colour |

`jailsp` and the browser playground always use plain text. Layout is separate from colour (errors red, warnings yellow, notes cyan, helps green).

### Warnings

`Compiler::warn` records a warning once per span and message; every `jaic` mode prints them before any error and they never change the exit status. `sema/flow.rs` reads procedure body shape for two of them: ``not all control paths return a value`` (a `return` counts, an `if` only with a returning `else`, a `switch` only with a default or `#complete`; `while true {}` warns, `while cond { return x; }` does not, as in Jai) and ``statements in block after `return` are never reached``. It runs from `lower_proc_body` and does not evaluate conditions.

### Format string check

`check_format_call` (`sema/format_check.rs`) runs in `call_procs` after overload resolution. A callee is print-like when a `string` parameter is directly followed by `..Any` and its body passes both to one call (`print`, `sprint`, `tprint`, `log`, `assert` messages, user wrappers). It warns when a literal format string's directive count differs from the arguments passed (`print("% is %\n", name)`), worded like Jai. Skipped when the format is not a literal, the call spreads an array or names a variadic argument, or the callee has no body. Directive rules follow `__format_to_builder` in `stdlib/Basic/Print.jai`: `%`/`%0` next argument, `%N` argument N (a following `%` continues after it), `%00` none, `%%` two arguments, `\%` a literal percent. `jaic::format_string` is the one reader of the directives (the compile check, the language server's hover and the linter use it); keep it in step with that function. The warning carries a fix when it is plain: extra arguments are removed (`drop_extra_arguments`, only when the call ends right after them and the removed text has balanced brackets), and a bare `%` with no argument is written `\%` (`escape_unmatched`, only when the escaped string then asks for exactly the arguments passed). Both are `MaybeIncorrect`: the author may have meant to add an argument or a `%` instead. The unreachable-code warning offers deleting the dead statements (never preferred), and `not all control paths return a value` notes the closing `}` of the body; neither guesses a value to return. Tests: `format_string_argument_count_is_checked` in `crates/jaic-cli/tests/diagnostics.rs`, `crates/jaic/tests/warning_hints.rs` (each edit is applied and recompiled).

### Runtime failures (`sema/trap_report.rs`)

When interpreted code fails a check (bounds, narrowing cast, `#complete` switch, null pointer, stack overflow, divide by zero, missing return, `#asm` fault, assertion), the interpreter returns a `Trap` with the message, statement and unwound procedures (`Trap::frames`, capped at 64). `Compiler::trap_diagnostic` makes the primary location the innermost frame in user code, adds `note: call stack (innermost first):` and a `help:` for common checks:

```
bounds.jai:4:5: error: runtime error: array bounds check failed: index 3 is outside an array of 3 elements
note: call stack (innermost first):
    `get` at bounds.jai:4
    `main` at bounds.jai:9
help: valid indices are 0 up to the array's count minus one; check the index or the array's length first
```

Messages come from one place, `ir::check_message`, keyed by `ir::CheckReason`. In a built executable a failed check calls the runtime's file-private `__jaic_check_failed` (resolved into `ir::Program::check_failed` by `note_check_handler`; checks go through `emit_check_failed`/`emit_trap`), printing the same message as `path:line: error: <message>`, then trapping. A `.NONFATAL` check prints `warning:` and continues in both backends. A metaprogram's own error (`compiler_report`, `.FAILED` status) sets `Trap::reported` and is shown as written.

Assertions: `context.assertion_failed` is invoked; the selected runtime can mark its default callback with the private `__jaic_interpreter_assertion` alias so the interpreter reports the assertion at the user's location (`Compiler::prepare_interpreter_assertion`, `Hook::AssertionFailed`, `Trap::assertion`). Custom callbacks keep their normal behavior.

#### Crashes in native code (`interp/crash.rs`)

`jaic -interp` and `#run` call foreign procedures in jaic's own process. While one runs, `crash::ForeignCall` publishes the symbol, the thread's interpreter state and the program in a thread-local; a fault handler (`sigaction` on an alternate stack; a vectored exception handler on Windows) prints the foreign call, the call stack and a help, then exits with status 121. The report is written into a fixed buffer with `write(2)` because the crash may be inside `malloc`, so it has no source excerpt. A fault on a thread with no foreign call in progress is not the handler's: it restores the previous action and returns. Tests: `crash_in_native_code_names_the_foreign_call` in `crates/jaic-cli/tests/diagnostics.rs`, `crash_in_native_code_is_reported_on_windows` in `tests/native.rs`.

### Command-line errors

`CliError` (message + helps): unknown commands and options get the closest match, missing input files suggest `suggest::similar_sibling`, build problems (output path is a directory, no `main`, unknown sanitizer) are caught before the backend, and a missing linker says how to install one. A Rust panic prints `error: internal compiler error: ...` with the issue tracker link (`install_panic_hook`).

### Exit status

`jaic`: 0 success (`-interp`: the program's own status), 1 compile/runtime/build/link error or unreadable input, 2 command-line mistake, 3 no `main`, 120 memory limit (`JAIC_MEMORY_LIMIT`), 121 native code crashed under the interpreter, 101 internal error. `jailint`: 1 for a `deny` finding, 2 for a command-line or `jailint.toml` mistake. `jaifmt`: 1 when `--check` finds files that would change.

## How to change it

- **New error**: build the `Diagnostic` at the narrowest span the user wrote; add `with_help` when the fix is clear and `with_note(span, ..)` for a related place. Add a case to `crates/jaic-cli/tests/diagnostics.rs`.
- **Rewording a message** breaks substring checks in `tests/corpus/manifest.json` (`python3 tools/differential/jaic_sweep.py negative`), `tests/compiler/stdlib-targets.txt`, `parser/tests.rs`, `crates/jaic-cli/tests/cli.rs`, language server tests and docs. Code never matches message text: sema reacts to `Diagnostic::kind`.
- **Errors other code reacts to** get a `DiagnosticKind` (`source.rs`, `with_kind`); `kind.code()` is the stable name the playground carries as `code`.
- **New runtime check**: add an `ir::CheckReason` variant with wording in `ir::check_message` (and the same in `runtime_support_check_failed` for executables), raise it with `Interp::check_trap`, and add a help in `help_for` if common. Notes and helps key off `Trap::kind`, never message text.
- **Layout/colour**: `render.rs`, tests in `render/tests.rs`. The plain layout must stay byte-for-byte stable (`tests/lint/*/bad.expected` depend on it).
- A command-line tool should call `jaic::render::set_style(detect(choice))` once and print `Report`s. Code rendering for a caller with its own style (the playground's `run_with`) uses `render::with_style(style, || ...)`.

## Configuration

| Setting | Effect |
|---|---|
| `--color auto\|always\|never` (`jaic`, `jailint`, `jaifmt`) | colour; `auto` colours terminals |
| `NO_COLOR` | no colour under `auto` |
| `CLICOLOR_FORCE`, `FORCE_COLOR` (set, not `0`) | colour even when not a terminal |
| `TERM=dumb` | plain layout, no colour |
| `JAIC_DIAGNOSTICS=plain\|ascii\|unicode` | force a layout |
| `LC_ALL` / `LC_CTYPE` / `LANG` with `UTF-8`, or `WT_SESSION` | `unicode` layout on a terminal |

## Dependencies

Standard library only: `render.rs`, `suggest.rs` (edit distance), `sema/suggestions.rs`, `sema/trap_report.rs`, `interp::Trap`, `stdlib/Runtime_Support/module.jai` (assertion reports in executables).
