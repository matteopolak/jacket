# Polymorphism and baking

Polymorphic procedures and structs (`$T`, `$N`, `$T/Constraint`) are instantiated once per distinct set of constant arguments {#poly.1}. `#bake_arguments` makes a new procedure or struct with some parameters fixed.

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

The results in the comments: `$T` inferred from an argument {#poly.2}, an explicit `$T: Type` {#poly.3}, `[] $T` from an array {#poly.4}, `$R` from a lambda's result {#poly.5}, and a constant `$N` {#poly.6}.

## Inference and restrictions

- A constant parameter with a declared bool, number or enum type (`$N: s32`) takes its argument as a runtime parameter does, so `f(true)` or `f("x")` is an error.
- A constant parameter's type may name an earlier parameter's binding: `($c: $C, $e: C.E)` reads `C.E` once `C` is bound.
- Struct parameters can bind variables inside their type (`struct (x: [$N] $T)`); the bound variables join the instance key {#poly.12}. They may be variadic (`struct(types: ..Type)`).
- `$T/Entity` matches `Entity` or a struct with it as an `#as` base {#poly.13}. A failed match lets other overloads win {#poly.14}. On an array or slice element, `$T` takes any element type {#poly.21}.
- A `$T/.[A, B]` restriction (or `Name :: Type.[A, B]`) admits only the listed types.
- A parameter typed with a bare polymorphic struct (`r: *Reflector`) makes the procedure polymorphic and accepts any instance {#poly.15}. A struct holding an instance as an `#as` member also matches and keeps its own type inside the body {#poly.16}.
- `$$x` is baked when the argument (or default) is constant and stays a runtime parameter otherwise, so `#if x` works in the body: `skip :: (p: *$T, $$must := false) { #if must ... }` {#poly.17}.
- An `ifx` argument binds `$T` from its first branch with a type of its own.

## #modify

A `#modify` block runs at compile time once per polymorph, before the instance exists. Type variables are assignable `Type` values (unbound ones start as `void`); baked `$x` values and inferred value variables (`$N`, `$C`) are constants. Whatever it leaves in a type variable becomes the binding, even a `$R` that appears only in the return type. `return false, "why"` rejects the call {#poly.18}.

- Inside the block a type variable is a runtime value, so use `(cast(*Type_Info) V).runtime_size` instead of `size_of(V)`.
- A `#modify` on something with no polymorph variables is an error at the `#modify`, called or not, since it would never run {#poly.19}.
- A procedure whose only polymorph-like parameters are `$$x` is accepted: the block runs for calls that bake `x`, not for runtime calls {#poly.20}.
- jaic also lets the block read inferred value variables (`N`).

## #bake_arguments

- Named parameters are bound; the result takes the rest in order {#poly.7}. Baking a struct leaves a struct taking the remaining parameters {#poly.8}.
- The target must be a single procedure or polymorphic struct {#poly.9}; extra arguments give "too many arguments to #bake_arguments" {#poly.10}.
- Baked struct instances are instances of the origin, so a `$V/Vec3` restriction accepts only matching ones {#poly.11}.

## How to change it

Instances are created by `instantiate` in `sema/procs.rs` (structs: `instantiate_struct` in `sema/structs.rs`); new inference rules go in `sema/calls/infer.rs`; baking is `check_bake` in `sema/bake.rs`. Type info lists baked parameters first in a struct's members; keep that order. Tests: `tests/compiler/regressions/modify-polymorph-forms.jai`, `baked-struct-restriction.jai`, `lang-poly-misc.jai`.
