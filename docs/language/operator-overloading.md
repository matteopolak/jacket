# Operator overloading

## What it is

`operator +`, `==`, `[]`, `*[]`, `[]=` and the rest, declared as procedures on user types. Binary, unary and index operators dispatch through ordinary overload resolution.

## How it works

`operator_candidates` in `sema/calls.rs` looks up the name `operator<text>` and filters candidates by operand type; `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Overloads travel with the operand's struct type, so a module's `operator []` (say, `Bit_Array`'s) works wherever the type is used.

```jai
Vec :: struct { x, y: float; }
operator +  :: (a: Vec, b: Vec) -> Vec { return .{a.x+b.x, a.y+b.y}; }
operator == :: (a: Vec, b: Vec) -> bool { return a.x==b.x && a.y==b.y; }
operator [] :: (v: Vec, i: int) -> float { if i == 0 return v.x; return v.y; }
```

Fallbacks:

- `a != b` becomes `!(a == b)` when no `operator !=` matches.
- `x[i]` uses `operator []` if one matches, else dereferences `operator *[]`, which also makes it assignable.
- `g[i] = v` calls `operator []=`.

`overloadable` decides which operand types may use overloads: structs, arrays, pointers, enums and distinct types. Plain numbers and bools always use the builtin operators.

`enum_flags` values have builtin `&`, `|` and `~` that keep the enum type, and compare with a literal zero:

```jai
Flags :: enum_flags u16 { READ :: 1; WRITE :: 2; }
f: Flags = .READ; f |= .WRITE;
f & .WRITE != 0        // true
(f & ~.READ) == .WRITE // true
```

## How to change it

For a new operator token, make the parser accept `operator <tok>` as a declaration, then add the dispatch next to the `try_*_operator_overload` functions. Tests: `tests/stdlib/operator-ne-from-eq.jai`, `tests/stdlib/index-operator-fallback.jai`.

## Dependencies

`sema/calls.rs`.
