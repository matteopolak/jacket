# Declarations, constants and globals

## What it is

How variables, constants, aliases, local types and nested procedures are declared and resolved, at file scope and inside procedures.

## How it works

Constants (`::`) are order independent. `check_block_stmts` in `sema/stmt.rs` hoists a block's constants before its statements, and file-level names resolve on demand (`sema/scope.rs`) {#decl.1}. Variables (`:=`, `: T =`) are ordinary left-to-right declarations {#decl.2}.

```jai
LIMIT :: 4;
SCALE : float : 2.5;                  // typed constant
Word :: u16;  answer : Word : 42;     // alias as the annotation
counter := 10;                        // mutable global
table: [LIMIT] int;                   // constant as an array size
squares := #run make_squares();       // initial value computed at compile time
```

Each form above compiles and has the value it suggests {#decl.3}.

- Local types, constants and procedures can refer to each other in any order: `Count :: Later + 1; Later :: 2; Local :: struct { values: [Count] int; }` works inside `main` {#decl.4}.
- A struct body can hold constants and procedures (`Node.LIM`, `Node.read`) {#decl.5}. `TRUE : s32 : 1;` inside a struct is a constant member, so it is not in `type_info(Rec).members` or `size_of` {#decl.6}.
- `x: int = ---;` leaves `x` uninitialised {#decl.7}; `a, b := 1, 2.5;` declares several at once {#decl.8}; struct fields with defaults (`v := 5;`) initialise on declaration {#decl.9}.
- Assigning to a constant fails: `cannot assign to constant 3 of type s64` {#decl.10}.
- A nested procedure sees constants, types and globals of its enclosing scope, not its locals {#decl.11}: `cannot access local 'local' of an enclosing procedure` {#decl.12}. It is a plain procedure, not a closure.
- `#run` initialisers run at compile time {#decl.13}. Changes other compile-time code makes to a global do not reach the running program unless the global is `#no_reset` (`tests/stdlib/compile-time-globals-reset.jai`) {#decl.14}.
- `a, b :: f();` binds every value of a multi-value constant, evaluated once per scope. `Compiler::multi_consts` is keyed by declaration and scope, so each macro expansion gets its own {#decl.15}. Naming more values than the expression has is an error (`3 names but 2 values`), while fewer names take the leading values {#decl.16}. `#run f()` keeps all of f's values {#decl.17}.
- `#assert cond "message";` and `#assert(cond, "message");` run at compile time and report at the assertion {#decl.18}: `#assert failed: Rec must be 8 bytes` {#decl.19}.
- Notes after a field (`v : s32 = 9 @Hidden;`) appear in `type_info(T).members[i].notes` {#decl.20}; notes on the struct (`S :: struct @thing { ... }`) in `type_info(S).notes` {#decl.21}.

## How to change it

- Local declarations: `check_local_decl` and `declare_local_consts` in `sema/stmt.rs`.
- File and module declarations: `sema/modules.rs` and `sema/decls.rs`; struct members: `sema/structs.rs`.
- Constant evaluation: `sema/consteval.rs`.
- The "enclosing local" error comes from name lookup in `sema/expr.rs`. Supporting captures would need a closure representation, which the IR does not have.

Add a regression program under `tests/stdlib/` for new forms; `type-field-constant.jai`, `macro-body-constants.jai`, `modern-metaprogramming.jai` and `struct-level-notes.jai` are examples. See [scoping](scoping.md) for visibility and [structs](structs.md) for members.

## Dependencies

`sema/stmt.rs`, `sema/scope.rs`, `sema/modules.rs`, `sema/decls.rs`, `sema/structs.rs`, `sema/consteval.rs`.
