# Directives, notes, `#must` and `#discard`

## What it is

Source markers that change how a declaration or statement is treated: directive suffixes (`#run,stallable`), `#assert`, `#program_export`, `#must`, `#discard`, `@notes`.

## How it works

`parser/directive.rs` parses suffixes generically as `,name` or `,name(expr)`. `#run,stallable` behaves like `#run`. `#assert cond "msg";` and `#assert(cond, "msg");` fail at the condition. `#program_export` keeps a procedure as a root and sets its native symbol (`export_entities` in `sema/modules.rs`). `;` alone is an empty statement.

### Notes

A note is `@` plus every character up to whitespace: `@short=j`, `@range(1,64)`. A touching `;` or `{` belongs to the note, so member notes go after the semicolon (`a: int; @tag`). `@help("two words")` is an error. `@"?text"` is read as one string note; this deviates from the older reference build because programs in the corpus use it (`Tok::Note` in `lexer.rs`).

Notes are stored, never interpreted: `type_info(T).members[i].notes`, `type_info(S).notes`, `Code_Struct.notes`, `Code_Enum.notes` (`Type_Info_Enum` has none). Built in `sema/typeinfo.rs` and `sema/code_export.rs`.

### `#must` and `#discard`

```jai
only :: () -> int, bool #must { return 1, true; }
only();            // error: the result of `only` is marked #must and cannot be discarded
_, ok := only();   // fine

assert :: (cond: bool, #discard msg := "") { ... }   // msg is type-checked, never evaluated
```

`#must` is checked when code is generated, not during type checking, so a call passed as `Code` to a macro that never runs it is fine. `emit_call` records the call in `last_call_must`; `check_stmt` and local declarations call `check_must_used` (`sema/stmt.rs`). `_ := f()` counts as use. Calls through procedure values and operator overloads do not carry `#must`.

A `#discard` argument is only type-checked (`discarded_args`, `precheck_args_deferring` in `sema/calls.rs`), so `#discard x: $T` still infers `T`, and defaults are not evaluated. Using the parameter in the callee is an error (`entity_operand`, `sema/expr.rs`). `Basic.assert` uses it when `ENABLE_ASSERT` is false.

## How to change it

- New directive suffix: `parser/directive.rs`, then consume it where the directive is evaluated. New procedure flag: `HEADER_DIRECTIVES` or `OTHER_FLAGS` in `parser/procedure.rs`, read in `sema/procs.rs`.
- Code that drops call results must call `check_must_used` right after the call (it matches by span). New call forms must route arguments through `precheck_args_deferring`.
- A check that rejects programs needs a negative test in `tests/corpus/negative/` and an entry in `tests/corpus/manifest.json`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `parser/directive.rs` parses suffixes generically as `,name` or `,name(expr)` after a directive (`#run,stallable`, `#insert,scope(x)`). `#run,stallable` behaves like `#run` {#note.1}.
- `#assert cond "msg";` and `#assert(cond, "msg");` both take an optional message {#note.2}, and a failure is reported at the condition {#note.3}. `#must` after a result makes discarding that result a compile error {#note.6}.
- `#program_export` (optionally with a symbol name string) on a procedure keeps it as a root even if nothing calls it (`export_entities` in `sema/modules.rs`) {#note.4} and sets its native symbol name to the string or the procedure name (`sema/procs.rs`) {#note.5}. It matters for `jaic`.
- Notes (`@Name`) are stored, never interpreted {#note.11}:
- | after a struct member | `type_info(T).members[i].notes` {#note.7} |
- | on a struct or union (`S :: struct @thing {`) | `type_info(S).notes`, `Code_Struct.notes` {#note.8} |
- | on an enum (`enum_flags @Hi {`) | `Code_Enum.notes` only {#note.9}; `Type_Info_Enum` has no notes {#note.10} |
- `;` alone is a statement, so `if answer ;` is valid with an empty body {#note.12}.
- `#must` on a result makes discarding it a compile error {#must.1}. `#discard` on a parameter means the callee cannot use it {#must.2} and the caller never evaluates the argument (it is only type-checked) {#must.3}. That is how `Basic.assert` costs nothing when assertions are off {#must.4}.
- The parser sets `Return::must` per result (`-> int, bool #must` marks the second) {#must.5}. Each time `emit_call` in `sema/calls.rs` emits a call or expands a macro, it records the call's span, name and `#must` flags in `last_call_must`.
- The check happens when code is generated, not during type checking. That is deliberate: a call passed as `Code` to a macro that never runs it does not violate `#must` {#must.6}.
- `a := f();`: `check_must_used(span, names)`, so only results beyond the declared names may not be `#must` {#must.7}. `_ := f();` counts as using it {#must.8}.
- Calls through procedure values and operator overloads don't carry `#must`; the flag lives on the declaration {#must.9}.
- Since the argument is still type-checked, `#discard x: $T` infers `T` {#must.10} and type errors are still reported {#must.11}.
- Defaults (`#discard loc := #caller_location`) are not evaluated either {#must.12}.
