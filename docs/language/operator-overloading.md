# Operator overloading

`operator +`, `==`, `[]`, `*[]`, `[]=` and the rest are declared as procedures on user types. Binary, unary and index operators dispatch through ordinary overload resolution {#opov.1}.

```jai
Vec :: struct { x, y: float; }
operator +  :: (a: Vec, b: Vec) -> Vec { return .{a.x+b.x, a.y+b.y}; }
operator == :: (a: Vec, b: Vec) -> bool { return a.x==b.x && a.y==b.y; }
operator [] :: (v: Vec, i: int) -> float { if i == 0 return v.x; return v.y; }
```

With these, `a + b` {#opov.3}, `a == b` {#opov.4} and `v[0]` {#opov.5} call the overloads.

## How it works

- The compiler looks up `operator<text>` and filters by operand type. Operators obey ordinary lexical visibility and filtered re-exports. A type alias alone doesn't import its defining scope's overloads {#opov.2}.
- `a != b` becomes `!(a == b)` when no `operator !=` matches {#opov.6}.
- `x[i]` uses `operator []` if one matches, else dereferences `operator *[]` {#opov.7}, which also makes it assignable {#opov.8}.
- `g[i] = v` calls `operator []=` {#opov.9}. A value with no type of its own (`g[i] = .{1, 2};`, `g[i] = .NAME;`) is built as the setter's value parameter, which may name the container's own parameters (`value: S.T`) {#opov.12}.
- Only structs, arrays, pointers, enums and distinct types can be overloaded; numbers and bools always use builtins.
- `enum_flags` values have builtin `&`, `|` and `~` that keep the enum type {#opov.10}, and compare with a literal zero {#opov.11}.
- An untyped `.{...}` as one operand (`.{6.0, 6.0} * f`, `v += .{1, 2}`, `v == .{1, 2}`) is typed by the chosen overload, like a literal call argument. Two equally good overloads are an ambiguity error; with no overload of that name the literal still needs `Type.{...}`.

## Leniency: operators through a namespaced import

Operators are also looked up in the module that declares an operand's struct, so `u + v` works for `Math.Vector2` behind `Math :: #import "Math"`. Overloads in scope are tried first, so this never makes a call ambiguous. It warns (`operator-out-of-scope`, see [leniencies](leniencies.md)) and suggests `Math.operator +(u, v)` or `using Math;`. When nothing matches, the error points at an alias that has the overload, else at a stdlib module that declares it.

## How to change it

For a new operator token, make the parser accept `operator <tok>`, then add dispatch beside the `try_*_operator_overload` functions in `sema/calls/operators.rs` (`operator_candidates` does the lookup). Tests: `tests/compiler/regressions/operator-ne-from-eq.jai`, `index-operator-fallback.jai`, `crates/jaic/tests/operator_home_module.rs`.
