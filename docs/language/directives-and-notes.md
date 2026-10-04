# Statement directives, flags and notes

## What it is

Small source-level markers that change how a declaration or statement is treated: `#run,stallable`, `#assert`, `#program_export`, `#must`, `@notes`, and empty statements.

## How it works

Directive suffixes are parsed generically as `,name` or `,name(expr)` after the directive (`parser/directive.rs`); for example `#run,stallable` and `#insert,scope(x)`. `#run,stallable print("x\n");` behaves like `#run` (it printed `at compile time` during the check).

`#assert cond "msg";` and `#assert(cond, "msg");` both take an optional message. They are evaluated at compile time and a failure is reported at the condition: `error: #assert failed: Rec must be 8 bytes`. See [declarations and constants](declarations-and-constants.md).

`#program_export` (optionally with a string) on a procedure keeps it as a root even if nothing calls it (`export_entities` in `sema/modules.rs`) and records the native symbol name (`sema/procs.rs`: the string, or the procedure's own name). It is relevant to `jaic build`.

`#must` after a result (`-> int #must`, or on one of several results) makes discarding that result a compile error; see [must-and-discard.md](must-and-discard.md). `#discard` on a parameter is described there too.

Notes (`@Name`) after a struct member are stored and exposed as `type_info(T).members[i].notes`; notes on the struct itself (`S :: struct @thing { ... }`, also `union` and polymorphic structs) are exposed as `type_info(S).notes` (`sema/typeinfo.rs`). Enum notes (`enum_flags @Hi { ... }`) reach metaprograms as `Code_Enum.notes`, as struct notes reach `Code_Struct.notes` (`sema/code_export.rs`); `Type_Info_Enum` has no notes field. Unrecognised notes never change behavior.

An empty statement `;` is a real statement: `if answer ;` is valid and has an empty body.

## How to change it

Add a new directive suffix in `parser/directive.rs` and consume it where the directive is evaluated. Add a procedure flag by extending `HEADER_DIRECTIVES` (or `OTHER_FLAGS`) in `parser/procedure.rs` and reading it in `sema/procs.rs`. A new check that must reject a program belongs with a negative program under `tests/corpus/negative` (registered in `tests/corpus/manifest.json`; `python3 tools/jaic-sweep.py negative` runs them).

## Configuration

None.

## Dependencies

`parser/directive.rs`, `parser/procedure.rs`, `sema/procs.rs`, `sema/modules.rs`, `sema/consteval.rs` (for `#assert` and `#run`).
