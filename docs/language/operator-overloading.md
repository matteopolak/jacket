# Operator overloading

## What it is

`operator +`, `==`, `[]`, `*[]`, `[]=` and friends declared as procedures on user types. Binary, unary and index operators dispatch through ordinary overload resolution.

## How it works

`operator_candidates` in `crates/jaic/src/sema/calls.rs` looks up the name `operator<text>` and filters candidates by the operand types. `try_binary_operator_overload`, `try_unary_operator_overload` and `try_index_operator_overload` call it. Overloads travel with the operand struct types, so a module's `operator []` (for example `Bit_Array`'s) is visible wherever the type is used.

```jai
Vec :: struct { x, y: float; }
operator +  :: (a: Vec, b: Vec) -> Vec { return .{a.x+b.x, a.y+b.y}; }
operator == :: (a: Vec, b: Vec) -> bool { return a.x==b.x && a.y==b.y; }
operator [] :: (v: Vec, i: int) -> float { if i == 0 return v.x; return v.y; }
// Vec.{1,2} + Vec.{3,4} -> 4 6 ; the == above -> false ; Vec.{1,2}[1] -> 2
```

Fallbacks verified by regression programs:

- `a != b` becomes `!(a == b)` when no `operator !=` matches (`tests/stdlib/operator-ne-from-eq.jai`).
- `x[i]` uses a matching `operator []`, else dereferences `operator *[]`, which also makes it assignable (`tests/stdlib/index-operator-fallback.jai`).
- `operator []=` handles `g[i] = v`: with `g[i] = x * 2` stored by the overload, `g[1] = 5; g[1]` printed `10`.

Flag enums have builtin operators: `enum_flags` `&`, `|`, `~` keep the enum type, and `flags & .WRITE != 0` compares with a literal zero:

```jai
Flags :: enum_flags u16 { READ :: 1; WRITE :: 2; }
f: Flags = .READ; f |= .WRITE;
f & .WRITE != 0        // true
(f & ~.READ) == .WRITE // true
```

## How to change it

- New operator token: make sure the parser accepts `operator <tok>` as a declaration, then add the dispatch site next to the `try_*_operator_overload` functions.
- `overloadable` (`sema/calls.rs`) decides which operand types may use overloads: structs, arrays, pointers, enums and distinct types. Plain numeric and bool operands always use the builtin operators.

## Configuration

None.

## Dependencies

`sema/calls.rs`; `Basic`'s `print` in the examples.
