# Polymorphism and baking

## What it is

Polymorphic procedures and structs (`$T`, `$N`, `$T/Constraint`) are instantiated once per distinct set of constant arguments. `#bake_arguments` makes a new procedure or struct with some parameters fixed.

## How it works

`instantiate` (`sema/procs.rs`) creates instances; `sema/calls.rs` infers `$` variables (`header_poly_names`, `is_poly_var_arg`); `instantiate_for_proc_type` handles a polymorphic procedure assigned to a procedure type.

```jai
mx       :: (a: $T, b: T) -> T { if a > b return a; return b; }
mk       :: ($T: Type, v: $U) -> T { return cast(T) v; }
poly_ret :: (a: $A, f: (A) -> $R) -> R { return f(a); }

mx(3, 4)  mx(1.5, 0.5)     // 4 1.5
mk(float, 3)               // 3
poly_ret(4, x => x + 0.5)  // 4.5

widen    :: (x: $T) -> $R #modify { R = float64; } { return cast(R) x; }
positive :: ($n: int) -> int #modify { return n > 0, "n must be positive"; } { return n; }

add3    :: (a: int, b: int, c: int) -> int { return a*100 + b*10 + c; }
with_b  :: #bake_arguments add3(b = 5);          // with_b(1, 2) == 152
IntPair :: #bake_arguments Pair(T = int);        // Pair :: struct ($T: Type, $N: int)
```

- Struct parameters can bind variables inside their type (`struct (x: [$N] $T)`; `match_pattern` in `instantiate_struct`) and may be variadic (`struct(types: ..Type)`, as `Tagged_Union` is); the whole list keys the instance.
- `$T/Entity` matches `Entity` or a struct with it as an `#as` base (`has_as_base`); a failed match lets other overloads win. A parameter typed with a bare polymorphic struct (`r: *Reflector`) is polymorphic and accepts any instance, including a struct holding one as an `#as` member.
- `$$x` is baked when the argument (or default) is constant, else stays a runtime parameter, so `#if x` works in the body.
- An `ifx` argument binds `$T` from its first branch with a type of its own (`ifx_binding_operand`).
- `#modify` runs at compile time once per polymorph with the variables in scope (type variables assignable, baked values constant); whatever it leaves in a type variable becomes the binding, and `return false, "why"` rejects the call. Deviation: jaic also lets it read inferred value variables (`N`), which Jai rejects. Inside the block a type variable is a runtime value, so use `(cast(*Type_Info) V).runtime_size` instead of `size_of(V)`. A `#modify` on something with no polymorph variables is an error (it would never run); a procedure whose only polymorph-like parameters are `$$x` is accepted, and the block runs only for baked calls.
- `check_bake` (`sema/bake.rs`) binds named parameters; the result takes the rest in order. The target must be a single procedure or polymorphic struct, and extra arguments give "too many arguments to #bake_arguments". For structs, `sema/structs.rs` records the baked constants and origin, so baked instances are instances of the origin (a `$V/Vec3` restriction accepts only matching ones).

## How to change it

New inference rules go in `calls.rs`; instance creation is `instantiate` in `procs.rs`. `sema/typeinfo.rs` lists baked parameters first in a struct's type info; keep that order. Tests: `tests/stdlib/modify-polymorph-forms.jai`, `struct-modify.jai`, `baked-struct-restriction.jai`, `variadic-struct-parameter.jai`, `poly-infer-from-ifx.jai`, `lang-poly-misc.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Polymorphic procedures and structs (`$T`, `$N`, `$T/Constraint`) are instantiated once per distinct set of constant arguments {#poly.1}. `#bake_arguments` makes a new procedure or struct with some parameters fixed.
- These give the commented results: `$T` inferred from an argument {#poly.2}, an explicit `$T: Type` {#poly.3}, `[] $T` from an array {#poly.4}, `$R` from a lambda's result {#poly.5} and a constant `$N` {#poly.6}.
- Struct parameters can declare type variables inside their type: `struct (x: $T)`, `struct (x: [$N] $T)`. `instantiate_struct` matches the argument type against the pattern (`match_pattern`) and adds the bound variables to the instance key {#poly.12}.
- `$T/Entity`, where `Entity` is a plain struct, matches only `Entity` or a struct with `Entity` as an `#as` base (`has_as_base`) {#poly.13}. A failed match lets other overloads win {#poly.14}.
- A parameter typed with a bare polymorphic struct (`r: *Reflector`) makes the procedure polymorphic and accepts any instance {#poly.15}. A struct holding an instance as an `#as` member also matches and keeps its own type inside the body, so its other members stay reachable {#poly.16}.
- A `$$x` parameter is baked when its argument is a constant and stays a runtime parameter otherwise. An omitted argument takes the default, which is baked when it is a constant, so `#if x` works in the body: `skip :: (p: *$T, $$must := false) { #if must ... }` called as `skip(p)` {#poly.17}.
- A `#modify` block runs at compile time once per polymorph, before the instance is made, with the polymorph variables in scope: type variables as assignable `Type` values (unbound ones start as `void`) and baked `$x` values, including value variables inferred from an argument type (`$N` in `Vec($T, $N)`, `$C` in `[$C] int`), as constants. Whatever it leaves in a type variable becomes the binding, so it can bind a `$R` that appears only in the return type; `return false` (or `return false, "why"`) rejects the call {#poly.18}:
- A `#modify` on a procedure with no polymorph variables (no `$T`, no `$x`, no parameter typed with a bare polymorphic struct), or on a struct without parameters, is an error at the `#modify`, whether or not the procedure is called: such a procedure is never instantiated, so the block would never run {#poly.19}:
- A procedure whose only polymorph-like parameters are `$$x` is accepted: a call passing a constant bakes `x` and the block runs for that instance with `x` in scope, while a call passing a runtime value uses the procedure as written and does not run the block {#poly.20}. Rejecting it would forbid a block that is useful for the baked calls; running it for runtime calls would leave it nothing to read.
- `check_bake` in `sema/bake.rs` binds named parameters; the result takes the remaining ones in order {#poly.7}:
- Baking a struct leaves a struct that takes the remaining parameters {#poly.8}.
- The target must be a single procedure or a polymorphic struct {#poly.9}; extra arguments give "too many arguments to #bake_arguments" {#poly.10}. For structs, `sema/structs.rs` records the baked constants and the origin, so instances of the baked struct are instances of the origin {#poly.11}; a `$V/Vec3` restriction where `Vec3` is baked accepts only matching instances.
