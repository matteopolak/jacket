# Operator overloading

## What it is

`operator +`, `==`, `[]`, `*[]`, `[]=` and the rest, declared as procedures on user types. They dispatch through ordinary overload resolution.

## How it works

`operator_candidates` in `sema/calls.rs` looks up `operator<text>` and filters by operand type; `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Operators follow normal lexical visibility and filtered reexports; a type alias alone does not import the defining scope's overloads.

```jai
Vec :: struct { x, y: float; }
operator +  :: (a: Vec, b: Vec) -> Vec { return .{a.x+b.x, a.y+b.y}; }
operator == :: (a: Vec, b: Vec) -> bool { return a.x==b.x && a.y==b.y; }
operator [] :: (v: Vec, i: int) -> float { if i == 0 return v.x; return v.y; }
```

- `a != b` becomes `!(a == b)` when no `operator !=` matches.
- `x[i]` uses `operator []`, else dereferences `operator *[]` (which is also assignable); `g[i] = v` calls `operator []=`.
- `overloadable` limits overloads to structs, arrays, pointers, enums and distinct types; numbers and bools always use builtins.
- `enum_flags` have builtin `&`, `|`, `~` that keep the enum type and compare with literal `0`.

## How to change it

For a new operator token, make the parser accept `operator <tok>`, then add dispatch next to the `try_*_operator_overload` functions in `sema/calls.rs`. Tests: `tests/stdlib/operator-ne-from-eq.jai`, `index-operator-fallback.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `operator +`, `==`, `[]`, `*[]`, `[]=` and the rest, declared as procedures on user types. Binary, unary and index operators dispatch through ordinary overload resolution {#opov.1}.
- `operator_candidates` in `sema/calls.rs` looks up the name `operator<text>` and filters candidates by operand type; `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Operators obey ordinary lexical visibility and filtered reexports. A canonical type alias alone does not import its defining scope's overloads {#opov.2}.
- With these, `a + b` {#opov.3}, `a == b` {#opov.4} and `v[0]` {#opov.5} on `Vec` values call the overloads.
- `a != b` becomes `!(a == b)` when no `operator !=` matches {#opov.6}.
- `x[i]` uses `operator []` if one matches, else dereferences `operator *[]` {#opov.7}, which also makes it assignable {#opov.8}.
- `g[i] = v` calls `operator []=` {#opov.9}.
- `enum_flags` values have builtin `&`, `|` and `~` that keep the enum type {#opov.10}, and compare with a literal zero {#opov.11}:
