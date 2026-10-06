# Statement directives, flags and notes

## What it is

Source markers that change how a declaration or statement is treated: directive suffixes like `#run,stallable`, `#assert`, `#program_export`, `#must`, `@notes`, and the empty statement.

## How it works

`parser/directive.rs` parses suffixes generically as `,name` or `,name(expr)` after a directive (`#run,stallable`, `#insert,scope(x)`). `#run,stallable` behaves like `#run`.

`#assert` is covered in [declarations and constants](declarations-and-constants.md); `#must` and `#discard` in [must and discard](must-and-discard.md).

`#program_export` (optionally with a symbol name string) on a procedure keeps it as a root even if nothing calls it (`export_entities` in `sema/modules.rs`) and sets its native symbol name to the string or the procedure name (`sema/procs.rs`). It matters for `jaic build`.

Notes (`@Name`) are stored, never interpreted:

| Where | Exposed as |
| --- | --- |
| after a struct member | `type_info(T).members[i].notes` |
| on a struct or union (`S :: struct @thing {`) | `type_info(S).notes`, `Code_Struct.notes` |
| on an enum (`enum_flags @Hi {`) | `Code_Enum.notes` only; `Type_Info_Enum` has no notes |

Type info comes from `sema/typeinfo.rs`, code nodes from `sema/code_export.rs`.

`;` alone is a statement, so `if answer ;` is valid with an empty body.

## How to change it

- New directive suffix: `parser/directive.rs`, then consume it where the directive is evaluated.
- New procedure flag: `HEADER_DIRECTIVES` or `OTHER_FLAGS` in `parser/procedure.rs`, read in `sema/procs.rs`.
- A new check that rejects programs needs a negative test in `tests/corpus/negative/`, registered in `tests/corpus/manifest.json` and run by `tools/jaic-sweep.py negative`.

## Dependencies

`parser/directive.rs`, `parser/procedure.rs`, `sema/procs.rs`, `sema/modules.rs`, `sema/consteval.rs`.
