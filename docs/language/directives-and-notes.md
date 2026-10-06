# Statement directives, flags and notes

## What it is

Source markers that change how a declaration or statement is treated: directive suffixes like `#run,stallable`, `#assert`, `#program_export`, `#must`, `@notes`, and the empty statement.

## How it works

`parser/directive.rs` parses suffixes generically as `,name` or `,name(expr)` after a directive (`#run,stallable`, `#insert,scope(x)`). `#run,stallable` behaves like `#run` {#note.1}.

`#assert cond "msg";` and `#assert(cond, "msg");` both take an optional message {#note.2}, and a failure is reported at the condition {#note.3}. `#must` after a result makes discarding that result a compile error {#note.6}.

`#assert` is covered in [declarations and constants](declarations-and-constants.md); `#must` and `#discard` in [must and discard](must-and-discard.md).

`#program_export` (optionally with a symbol name string) on a procedure keeps it as a root even if nothing calls it (`export_entities` in `sema/modules.rs`) {#note.4} and sets its native symbol name to the string or the procedure name (`sema/procs.rs`) {#note.5}. It matters for `jaic build`.

Notes (`@Name`) are stored, never interpreted {#note.11}:

| Where | Exposed as |
| --- | --- |
| after a struct member | `type_info(T).members[i].notes` {#note.7} |
| on a struct or union (`S :: struct @thing {`) | `type_info(S).notes`, `Code_Struct.notes` {#note.8} |
| on an enum (`enum_flags @Hi {`) | `Code_Enum.notes` only {#note.9}; `Type_Info_Enum` has no notes {#note.10} |

Type info comes from `sema/typeinfo.rs`, code nodes from `sema/code_export.rs`.

`;` alone is a statement, so `if answer ;` is valid with an empty body {#note.12}.

## How to change it

- New directive suffix: `parser/directive.rs`, then consume it where the directive is evaluated.
- New procedure flag: `HEADER_DIRECTIVES` or `OTHER_FLAGS` in `parser/procedure.rs`, read in `sema/procs.rs`.
- A new check that rejects programs needs a negative test in `tests/corpus/negative/`, registered in `tests/corpus/manifest.json` and run by `tools/jaic-sweep.py negative`.

## Dependencies

`parser/directive.rs`, `parser/procedure.rs`, `sema/procs.rs`, `sema/modules.rs`, `sema/consteval.rs`.
