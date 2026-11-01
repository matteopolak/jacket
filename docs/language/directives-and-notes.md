# Directives, notes, `#must` and `#discard`

Source markers that change how a declaration or statement is treated: directive suffixes (`#run,stallable`), `#assert`, `#program_export`, `#must`, `#discard`, `@notes`.

## Directives

- Suffixes are written `,name` or `,name(expr)` after a directive (`#run,stallable`, `#insert,scope(x)`). `#run,stallable` behaves like `#run` {#note.1}.
- `#assert cond "msg";` and `#assert(cond, "msg");` both take an optional message {#note.2}; a failure is reported at the condition {#note.3}.
- A `#assert` whose condition holds a braced body (`#assert #run -> bool { ... } "msg"`) needs no `;`; any other does {#note.13}.
- `#program_export` (optionally with a symbol string) keeps a procedure as a root even if nothing calls it {#note.4} and sets its native symbol name to the string or the procedure name {#note.5}.
- `;` alone is an empty statement, so `if answer ;` is valid {#note.12}.

## Notes

A note is `@` plus every character up to whitespace: `@short=j`, `@range(1,64)`. A touching `;` or `{` belongs to the note, so member notes go after the semicolon (`a: int; @tag`). `@help("two words")` is an error. `@"?text"` is one string note.

Notes are stored, never interpreted {#note.11}:

| Where | Read from |
|---|---|
| after a struct member | `type_info(T).members[i].notes` {#note.7} |
| on a struct or union (`S :: struct @thing {`) | `type_info(S).notes`, `Code_Struct.notes` {#note.8} |
| on an enum (`enum_flags @Hi {`) | `Code_Enum.notes` only {#note.9}; `Type_Info_Enum` has no notes {#note.10} |

The notes on procedures the compiler reads are `@PrintLike` (checks a format string against its arguments), `@Interpolates` ([inline holes](interpolates.md)) and `@Cold` (the procedure runs only when something has gone wrong, such as a failed assertion: native builds compile it, and code only it reaches, without optimization to save build time; behaviour is the same).

## `#must` and `#discard`

```jai
only :: () -> int, bool #must { return 1, true; }
only();            // error: the result of `only` is marked #must and cannot be discarded
_, ok := only();   // fine

assert :: (cond: bool, #discard msg := "") { ... }   // msg is type-checked, never evaluated
```

- `#must` on a result makes discarding it a compile error {#note.6} {#must.1}. It is per result: `-> int, bool #must` marks the second {#must.5}.
- The check runs when code is generated, not during type checking. A call passed as `Code` to a macro that never runs it doesn't violate `#must` {#must.6}.
- `a := f();` uses only the declared names, so results beyond them may not be `#must` {#must.7}. `_ := f();` counts as use {#must.8}.
- Calls through procedure values and operator overloads don't carry `#must`; the flag lives on the declaration {#must.9}.
- `#discard` on a parameter means the callee can't use it {#must.2} and the caller never evaluates the argument {#must.3}. That is how `Basic.assert` costs nothing when assertions are off {#must.4}.
- The argument is still type-checked, so `#discard x: $T` infers `T` {#must.10} and type errors are reported {#must.11}. Defaults (`#discard loc := #caller_location`) aren't evaluated either {#must.12}.

## `#target_features`

```jai
sum_avx2 :: (xs: [] u8) -> u64 #target_features "avx2,bmi2" { ... }
```

Native code for the procedure may use the listed CPU features on top of the build's. A leading `-` removes one. Names the target doesn't have are dropped, so one source can list x64 features and still build for arm64 or wasm. The interpreter ignores it. LLVM inlines a procedure only into callers with at least its features, so calling it from baseline code is a real call, which is what a dispatch table wants ([SIMD](../extensions/simd.md)). Vector intrinsics still lower for the build's features, not the procedure's.

## How to change it

- New expression directive: a `Directive` variant in `parser/directive.rs` and an arm in `parse_directive_expr`. New procedure flag: a `ProcFlag` variant and an arm in `apply_proc_flag` (`parser/procedure.rs`), read in `sema/procs.rs`.
- Code that drops call results must call `check_must_used` right after the call. New call forms must route arguments through `precheck_args_deferring`.
- A check that rejects programs needs a negative test in `tests/corpus/negative/` and a manifest entry.
