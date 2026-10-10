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
- A scope declares each name once: a second file-level, struct-level or block-level constant, global or named import with a name already taken is ``error: `X` is already declared in this scope`` with a note at the first {#decl.24}, and so are two parameters of one procedure. Only procedures (and aliases of them, `print :: print_to_builder;`) share a name as an overload set, and a `#placeholder` may be defined by several added strings. Checked in `check_redeclared` (`sema/scope.rs`), called while the scope's statements are declared; a local variable that reuses a name keeps its own check.
- A struct body can hold constants and procedures (`Node.LIM`, `Node.read`) {#decl.5}. `TRUE : s32 : 1;` inside a struct is a constant member, so it takes no space in `size_of` {#decl.6}. `type_info(Rec).members` lists constants too, in declaration order between the fields, with the `CONSTANT` flag; a constant's value (a type's `*Type_Info`, a procedure's address) is at `offset_into_constant_storage` in the struct's `constant_storage`, and `-1` means it has none (`struct_constants` and `constant_member` in `sema/typeinfo.rs`) {#decl.23}. Reflection code skips them with `if it.flags & .CONSTANT continue;` (a constant's `offset_in_bytes` is `-1`). Every procedure constant is listed, used or not, so code can look a hook up by name; a procedure whose signature is not yet resolved gets the shape of its header as its type (each parameter and result `$`, its value still the procedure's address), because resolving the real types there could lay them out too early, as when Vk-Engine's entity methods take a `*World` whose `#insert` reads a list its metaprogram is still building. Once something has used the procedure its exact type is reported. Overload set aliases (`dot :: dot_product;`) are not listed. Vk-Engine's own `GetNonConstantStructMember` skips `CONSTANT` members, so Jai lists them too.
- `x: int = ---;` leaves `x` uninitialised {#decl.7}; `a, b := 1, 2.5;` declares several at once {#decl.8}; a mixed list declares some names and assigns to existing places, which may be members, array elements or dereferences: `ok:, t.str = f();` and `t.pos[0]=, rest := g();` (the `:` marks a new name before `=`, the `=` an existing place before `:=`) {#decl.22}; struct fields with defaults (`v := 5;`) initialise on declaration {#decl.9}.
- Assigning to a constant fails: `cannot assign to constant 3 of type s64` {#decl.10}.
- A nested procedure sees constants, types and globals of its enclosing scope, not its locals {#decl.11}: ``cannot access local `local` of an enclosing procedure`` {#decl.12}. It is a plain procedure, not a closure.
- `#run` initialisers run at compile time {#decl.13}. Changes other compile-time code makes to a global do not reach the running program unless the global is `#no_reset` (`tests/stdlib/compile-time-globals-reset.jai`) {#decl.14}.
- `a, b :: f();` binds every value of a multi-value constant, evaluated once per scope. `Compiler::multi_consts` is keyed by declaration and scope, so each macro expansion gets its own {#decl.15}. Naming more values than the expression has is an error (`3 names but 2 values`), while fewer names take the leading values {#decl.16}. `#run f()` keeps all of f's values {#decl.17}.
- `#assert cond "message";` and `#assert(cond, "message");` run at compile time and report at the assertion {#decl.18}: `#assert failed: Rec must be 8 bytes` {#decl.19}.
- Notes after a field (`v : s32 = 9 @Hidden;`) appear in `type_info(T).members[i].notes` {#decl.20}; notes on the struct (`S :: struct @thing { ... }`) in `type_info(S).notes` {#decl.21}. Notes on a struct-scoped constant (`K :: 3; @kn`) appear in that constant member's `notes`; `#no_padding` sets `textual_flags.NO_PADDING` and a struct whose members are all `= ---` sets `nontextual_flags.ALL_MEMBERS_UNINITIALIZED` (its `initializer` stays non-null and leaves the members untouched). A struct declared inside a procedure, a block or another struct has `status_flags.LOCAL`.

## How to change it

- Local declarations: `check_local_decl` and `declare_local_consts` in `sema/stmt.rs`.
- File and module declarations: `sema/modules.rs` and `sema/decls.rs`; struct members: `sema/structs.rs`.
- Constant evaluation: `sema/consteval.rs`.
- The "enclosing local" error comes from name lookup in `sema/expr.rs`. Supporting captures would need a closure representation, which the IR does not have.

Add a regression program under `tests/stdlib/` for new forms; `type-field-constant.jai`, `macro-body-constants.jai`, `modern-metaprogramming.jai` and `struct-level-notes.jai` are examples. See [scoping](scoping.md) for visibility and [structs](structs.md) for members.

## Dependencies

`sema/stmt.rs`, `sema/scope.rs`, `sema/modules.rs`, `sema/decls.rs`, `sema/structs.rs`, `sema/consteval.rs`.
