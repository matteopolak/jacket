# Polymorphism and baking

## What it is

Polymorphic procedures and structs (`$T`, `$N`, `$T/Constraint`) are instantiated once per distinct set of constant arguments. `#bake_arguments` makes a new procedure or struct with some parameters fixed.

## How it works

`instantiate` in `sema/procs.rs` creates instances. `sema/calls.rs` infers `$` variables from arguments (`header_poly_names`, `is_poly_var_arg`); `instantiate_for_proc_type` handles a polymorphic procedure assigned to a procedure type.

```jai
mx       :: (a: $T, b: T) -> T { if a > b return a; return b; }
mk       :: ($T: Type, v: $U) -> T { return cast(T) v; }
count_of :: (arr: [] $T) -> int { return arr.count; }
poly_ret :: (a: $A, f: (A) -> $R) -> R { return f(a); }
cn       :: ($N: int) -> int { return N * 2; }

mx(3, 4)  mx(1.5, 0.5)     // 4 1.5
mk(float, 3)               // 3
poly_ret(4, x => x + 0.5)  // 4.5
cn(21)                     // 42
```

Struct parameters can declare type variables inside their type: `struct (x: $T)`, `struct (x: [$N] $T)`. `instantiate_struct` matches the argument type against the pattern (`match_pattern`) and adds the bound variables to the instance key.

`$T/Entity`, where `Entity` is a plain struct, matches only `Entity` or a struct with `Entity` as an `#as` base (`has_as_base`). A failed match lets other overloads win.

### `#bake_arguments`

`check_bake` in `sema/bake.rs` binds named parameters; the result takes the remaining ones in order:

```jai
add3    :: (a: int, b: int, c: int) -> int { return a*100 + b*10 + c; }
with_b  :: #bake_arguments add3(b = 5);          // with_b(1, 2) == 152
with_ab :: #bake_arguments add3(a = 1, b = 2);   // with_ab(9)   == 129

Pair    :: struct ($T: Type, $N: int) { items: [N] T; }
IntPair :: #bake_arguments Pair(T = int);        // IntPair(3) has 3 items
```

The target must be a single procedure or a polymorphic struct. For structs, `sema/structs.rs` records the baked constants and the origin, so instances of the baked struct are instances of the origin; a `$V/Vec3` restriction where `Vec3` is baked accepts only matching instances.

## How to change it

New inference rules go in `calls.rs`; instance creation is `instantiate` in `procs.rs`. `sema/typeinfo.rs` lists baked parameters first in a struct's type info; keep that order.

Tests: `tests/stdlib/baked-default-uses-poly.jai`, `baked-struct-restriction.jai`, `baked-overload-runtime-field.jai`, `lang-poly-misc.jai`, `lang-lambdas.jai`.

## Dependencies

`crates/jaic/src/sema` only.
