# Polymorphism and baking

## What it is

Polymorphic procedures and structs (`$T`, `$N`, `$T/Constraint`) are instantiated per distinct constant argument set, and `#bake_arguments` creates a new procedure or struct with some parameters fixed.

## How it works

Instantiation is `instantiate` in `crates/jaic/src/sema/procs.rs`; `sema/calls.rs` infers `$` variables from arguments (`header_poly_names`, `is_poly_var_arg`) and `instantiate_for_proc_type` handles a polymorphic procedure assigned to a procedure type.

```jai
mx       :: (a: $T, b: T) -> T { if a > b return a; return b; }
mk       :: ($T: Type, v: $U) -> T { return cast(T) v; }
count_of :: (arr: [] $T) -> int { return arr.count; }
poly_ret :: (a: $A, f: (A) -> $R) -> R { return f(a); }
cn       :: ($N: int) -> int { return N * 2; }

mx(3, 4)  mx(1.5, 0.5)     // 4 1.5
mk(float, 3)               // 3
count_of(int.[1,2,3])      // 3
poly_ret(4, x => x + 0.5)  // 4.5
cn(21)                     // 42
```

`#bake_arguments` (`sema/bake.rs`, `check_bake`) binds named parameters; the baked procedure takes the remaining parameters in order:

```jai
add3    :: (a: int, b: int, c: int) -> int { return a*100 + b*10 + c; }
with_b  :: #bake_arguments add3(b = 5);          // with_b(1, 2) == 152
with_ab :: #bake_arguments add3(a = 1, b = 2);   // with_ab(9)   == 129

Pair    :: struct ($T: Type, $N: int) { items: [N] T; }
IntPair :: #bake_arguments Pair(T = int);        // p: IntPair(3) -> items.count == 3
```

The target must be a single procedure or a polymorphic struct (error: "#bake_arguments needs a single procedure or a polymorphic struct"); extra arguments give "too many arguments to #bake_arguments". For structs, `sema/structs.rs` records the baked constants and the origin struct, so instances of the baked struct are the origin's instances (see the comment "A `#bake_arguments` struct's instances" in `sema/calls.rs`).

- A polymorphic struct parameter may declare type variables in its type: `struct (x: $T)`, `struct (x: [$N] $T)`. `instantiate_struct` matches the argument's type against the pattern (`match_pattern`), binds the variables in the parameter scope and adds them to the instance key and bindings.
- `$T/Entity` with a non-polymorphic struct restriction matches only that struct or one with it as an `#as` base (`has_as_base` in `match_pattern`); a failed match lets other overloads (`$T/Strange`) win.

## How to change it

- Regression programs: `tests/stdlib/baked-default-uses-poly.jai`, `baked-struct-restriction.jai` (a `$V/Vec3` restriction where `Vec3` is a baked struct accepts only matching instances), `baked-overload-runtime-field.jai`, `lang-poly-misc.jai`, `lang-lambdas.jai`.
- New inference rules go in `calls.rs`; instance creation is `procs.rs::instantiate`.
- `sema/typeinfo.rs` lists baked parameters first in a struct's type info; keep that order if you touch it.

## Configuration

None.

## Dependencies

Semantic analysis only (`crates/jaic/src/sema`); no external libraries.
