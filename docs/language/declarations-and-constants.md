# Declarations, constants and globals

How variables, constants, aliases, local types, nested procedures and foreign data are declared and resolved. File-level constants (`::`) are order independent and resolve on demand; a constant in a body is visible only below its declaration (see [scoping](scoping.md)) {#decl.1}. Variables (`:=`, `: T =`) are ordinary left-to-right declarations {#decl.2}.

```jai
LIMIT :: 4;
SCALE : float : 2.5;                  // typed constant
counter := 10;                        // mutable global
table: [LIMIT] int;
squares := #run make_squares();       // computed at compile time
x: int = ---;                         // uninitialised
a, b := 1, 2.5;
ok:, t.str = f();                     // new name and an existing place
a, b :: f();                          // every value of a multi-value constant
#assert size_of(Rec) == 8 "Rec must be 8 bytes";
```

Each form compiles and has the value it suggests {#decl.3}.

## Rules

- Local types, constants and procedures can refer to ones declared above them: `Later :: 2; Count :: Later + 1; Local :: struct { values: [Count] int; }` works inside `main`; the other order is an error {#decl.4}.
- A scope declares each name once: ``error: `X` is already declared in this scope`` {#decl.24}. Two parameters of one procedure clash too. Only procedures and aliases of them (`print :: print_to_builder;`) form overload sets.
- Assigning to a constant fails: ``cannot assign to constant `N` `` {#decl.10}.
- A declaration can't take a builtin type's name (`int`, `float`, `bool`, `string`, `void`, `Any`, `Code`, `s8`..`u64`, `float32`, `float64`): `shadows-builtin-type` {#decl.25}. Enum members, `for` variables and `$T` names are fine; `Type`, `Context` and `Allocator` may be redeclared.
- `x: int = ---;` leaves `x` uninitialised {#decl.7}. `a, b := 1, 2.5;` declares several {#decl.8}. In a mixed list, `:` marks a new name before `=` and `=` marks an existing place (member, element, dereference) before `:=`: `ok:, t.str = f();` {#decl.22}. Struct fields with defaults initialise on declaration {#decl.9}.
- `x := call();` where the procedure returns nothing makes the call and declares a zero-size `void` variable {#decl.26}.
- A nested procedure sees enclosing constants, types and globals, not locals {#decl.11}: ``cannot access local `local` of an enclosing procedure`` {#decl.12}. There are no closures.
- `#run` initialisers run at compile time {#decl.13}. Changes other compile-time code makes to a global don't reach the program unless it is `#no_reset` {#decl.14}.
- `a, b :: f();` binds every value of a multi-value constant, evaluated once per scope (each macro expansion gets its own) {#decl.15}. More names than values is an error; fewer take the leading values {#decl.16}. `#run f()` keeps all of `f`'s values {#decl.17}.
- `#assert cond "message";` and `#assert(cond, "message");` run at compile time and report at the assertion {#decl.18}: `#assert failed: Rec must be 8 bytes` {#decl.19}.

## Struct-scoped constants and notes

- A struct body can hold constants and procedures (`Node.LIM`, `Node.read`) {#decl.5}. `TRUE : s32 : 1;` takes no space in `size_of` {#decl.6}.
- `type_info(Rec).members` lists them between the fields with the `CONSTANT` flag, `offset_in_bytes` of `-1`, and the value at `offset_into_constant_storage` in `constant_storage` (`-1` means none) {#decl.23}. Reflection code skips them with `if it.flags & .CONSTANT continue;`. Every procedure constant is listed, used or not; one whose signature isn't resolved yet gets a `$`-shaped header type so reflection doesn't force layout early. Overload-set aliases aren't listed.
- Notes after a field (`v : s32 = 9 @Hidden;`) appear in `type_info(T).members[i].notes` {#decl.20}; notes on the struct (`S :: struct @thing { ... }`) in `type_info(S).notes` {#decl.21}. Notes on a constant appear on that constant member.
- `#no_padding` sets `textual_flags.NO_PADDING`; a struct whose members are all `= ---` sets `nontextual_flags.ALL_MEMBERS_UNINITIALIZED`; a struct declared inside a procedure, block or struct has `status_flags.LOCAL`.

## Foreign data

`x: T #elsewhere lib "symbol";` declares a variable stored in a foreign library or the process {#extdata.1}. It has no initializer. The symbol defaults to the variable name {#extdata.2}; a string after the library overrides it {#extdata.3}. Without a library, only the compiler's own `__runtime_info` is recognised {#extdata.4}.

```jai
libc :: #library,system "libSystem";
out: *void #elsewhere libc "__stdoutp";
```

## How to change it

- Locals: `check_local_decl`, `declare_local_consts` (`sema/stmt.rs`). File level: `sema/modules.rs`, `sema/decls.rs`. Members: `sema/structs.rs`. Reflection: `struct_constants` in `sema/typeinfo.rs`.
- Redeclaration check: `check_redeclared` in `sema/scope.rs`.
- `#elsewhere`: `parse_elsewhere_library` in `parser/decl.rs`, `foreign_global` in `sema/decls.rs`. Library search uses `JAIC_NATIVE_LIBS` and the stdlib directory.

See [structs](structs.md).
