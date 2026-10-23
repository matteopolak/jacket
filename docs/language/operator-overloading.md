# Operator overloading

## What it is

`operator +`, `==`, `[]`, `*[]`, `[]=` and the rest, declared as procedures on user types. They dispatch through ordinary overload resolution.

## How it works

`operator_candidates` in `sema/calls/operators.rs` looks up `operator<text>` and filters by operand type; `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Operators follow normal lexical visibility and filtered reexports; a type alias alone does not import the defining scope's overloads.

```jai
Vec :: struct { x, y: float; }
operator +  :: (a: Vec, b: Vec) -> Vec { return .{a.x+b.x, a.y+b.y}; }
operator == :: (a: Vec, b: Vec) -> bool { return a.x==b.x && a.y==b.y; }
operator [] :: (v: Vec, i: int) -> float { if i == 0 return v.x; return v.y; }
```

- Operators are also looked up in the module that declares an operand's struct (`struct_home_scope`, as `for_expansion` is), so `u + v` works for `Math.Vector2` behind `Math :: #import "Math"`. Overloads in scope are tried first; the module's are tried only when none matches, so they never make a call ambiguous. Using one warns (`operator-out-of-scope`) and suggests `Math.operator +(u, v)`, the spelling that works anywhere, which `Compiler::warn_lenient_operator` in `sema/calls/lenient.rs` builds: the alias is the namespaced import in scope whose module exports the overload. `operator *[]` is looked up this way too (`pointer_index_reachable` in `sema/expr/indexing.rs`); as a write target or under `*`, only when the base names a variable, since the base type is needed first and checking an arbitrary base twice is not free; its warning offers only `using`, because `Alias.operator *[](..)` does not parse. A `.{...}` literal next to an operand whose module has the overload takes its type from that overload's parameter (`overload_typed_operands`). When nothing matches, `with_missing_operator_help` points at an alias that has the overload (`Alias.operator +(a, b)` or `using Alias;`), else at a standard-library module that declares it (`Suggestion::import`), but only one that is the home module of an operand's struct or has an overload whose parameters name every struct operand (`imports_declaring_operator`, a textual test); no qualifying module means no import suggestion. Tests: `crates/jaic/tests/operator_home_module.rs`.
- `a != b` becomes `!(a == b)` when no `operator !=` matches.
- An untyped `.{...}` as one operand (`.{6.0, 6.0} * f`, `v + .{1, 2}`, `v += .{1, 2}`, `v == .{1, 2}`) is typed by the overload chosen, as a literal call argument is: `overload_typed_operands` checks the other operand without emitting, ranks the overloads with the literal left open, and the operands are then checked once with the literal as the winning parameter type. The ranking is the one calls use ([types from context](casts-and-conversions.md#types-from-context)): two overloads that fit equally are an ambiguity error, and with no `operator` of that name the literal still needs `Type.{...}`. Tests: `tests/compiler/regressions/operator-literal-operands.jai`, `operator-literal-overloads.jai`, `crates/jaic/tests/operator_literal_operands.rs`.
- `x[i]` uses `operator []`, else dereferences `operator *[]` (which is also assignable); `g[i] = v` calls `operator []=`.
- `overloadable` limits overloads to structs, arrays, pointers, enums and distinct types; numbers and bools always use builtins.
- `enum_flags` have builtin `&`, `|`, `~` that keep the enum type and compare with literal `0`.

## How to change it

For a new operator token, make the parser accept `operator <tok>`, then add dispatch next to the `try_*_operator_overload` functions in `sema/calls/operators.rs`. Tests: `tests/compiler/regressions/operator-ne-from-eq.jai`, `index-operator-fallback.jai`.

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

- `operator +`, `==`, `[]`, `*[]`, `[]=` and the rest, declared as procedures on user types. Binary, unary and index operators dispatch through ordinary overload resolution {#opov.1}.
- `operator_candidates` in `sema/calls/operators.rs` looks up the name `operator<text>` and filters candidates by operand type; `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Operators obey ordinary lexical visibility and filtered reexports. A canonical type alias alone does not import its defining scope's overloads {#opov.2}.
- With these, `a + b` {#opov.3}, `a == b` {#opov.4} and `v[0]` {#opov.5} on `Vec` values call the overloads.
- `a != b` becomes `!(a == b)` when no `operator !=` matches {#opov.6}.
- `x[i]` uses `operator []` if one matches, else dereferences `operator *[]` {#opov.7}, which also makes it assignable {#opov.8}.
- `g[i] = v` calls `operator []=` {#opov.9}.
- `enum_flags` values have builtin `&`, `|` and `~` that keep the enum type {#opov.10}, and compare with a literal zero {#opov.11}:
