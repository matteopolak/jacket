# Declarations, constants and globals

## What it is

How variables, constants, aliases, local types and nested procedures are declared and resolved, at file scope and inside procedures.

## How it works

Constants (`::`) are order independent: a block's constants are hoisted before its statements run by `check_block_stmts` (`sema/stmt.rs`, "Constants are visible throughout their block, also before their declaration"), and file-level names are resolved on demand (`sema/scope.rs`). Variables (`:=`, `: T =`) are ordinary left-to-right.

```jai
LIMIT :: 4;
SCALE : float : 2.5;                  // typed constant
Word :: u16;  answer : Word : 42;     // alias used as the annotation
counter := 10;                        // global, mutable
table: [LIMIT] int;                   // constant in an array size
squares := #run make_squares();       // global initialised at compile time
```

Verified with `jaic run`:

- Local types, constants and procedures may refer to each other regardless of order: `Count :: Later + 1; Later :: 2; Local :: struct { values: [Count] int; }` works inside `main`.
- A struct body can hold constants and procedures: `Node.Count`, `Node.LIM`, and `Node.read` resolve as namespace members. `TRUE : s32 : 1;` inside a struct is a constant member, not a field (so it does not appear in `type_info(Rec).members` or `size_of`).
- `x: int = ---;` declares without initialising; `a, b := 1, 2.5;` declares several at once; struct fields with defaults (`v := 5;`) initialise on declaration.
- Assigning to a constant is an error: `cannot assign to constant 3 of type s64`.
- A nested procedure sees constants, types and globals of its enclosing scope but not its runtime locals: `cannot access local 'local' of an enclosing procedure` (`sema/expr.rs`). It is a plain procedure, not a closure.
- `squares := #run make_squares();` computes the initial value at compile time (it printed `[0, 1, 4, 9]`). Changes that other compile-time code makes to a global do not reach the running program unless the variable is `#no_reset`; see `tests/stdlib/compile-time-globals-reset.jai`.
- `a, b :: f();` binds every value of a multi-value constant expression, evaluated once per scope (`Compiler::multi_consts`, keyed by declaration and scope so each macro expansion gets its own); a count mismatch is `2 names but 3 values`. `#run f()` keeps all of f's values. See `tests/stdlib/modern-metaprogramming.jai`.
- `#assert cond "message";` and `#assert(cond, "message");` run at compile time, and a failure is reported at the assertion: `#assert failed: Rec must be 8 bytes`.
- Notes after a field (`v : s32 = 9 @Hidden;`) are visible through `type_info(T).members[i].notes`; the count was 1 for that field. Notes on the struct itself (`S :: struct @thing { ... }`) are `type_info(S).notes`; see `tests/stdlib/struct-level-notes.jai`.

## How to change it

Local declaration handling is in `check_local_decl` and `declare_local_consts` (`sema/stmt.rs`); file and module declarations are collected in `sema/modules.rs` and `sema/decls.rs`, with struct members in `sema/structs.rs`. Constant values are evaluated by `sema/consteval.rs`. The "enclosing local" diagnostic comes from name lookup in `sema/expr.rs`; supporting captures would need a real closure representation, which the IR does not have.

Add a regression program under `tests/stdlib/` for new declaration forms (constants in structs, `#if` members and the like have examples such as `tests/stdlib/type-field-constant.jai` and `tests/stdlib/macro-body-constants.jai`). See [scoping](scoping.md) for visibility rules and [structs](structs.md) for member declarations.

## Configuration

None.

## Dependencies

`sema/stmt.rs`, `sema/scope.rs`, `sema/modules.rs`, `sema/decls.rs`, `sema/structs.rs`, `sema/consteval.rs`.
