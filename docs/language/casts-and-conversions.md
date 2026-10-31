# Casts and conversions

How a value changes type: implicit conversion, `cast(T)`, the contextual `xx`, postfix `.(T)`, and the `no_check`, `trunc` and `force` modifiers.

```jai
small := cast,no_check(u8) wide;     // u16 298 -> 42
t := cast,trunc(u8) 256;             // 0
x: u8 = xx,trunc 298;                // 42, target from the declaration
p := (42).(u8);  q := big.(u32, trunc);   // postfix forms 
bits := cast,force(u32) f;           // reinterprets float bits
```

The values in the comments are what the example produces {#cast.1}; the postfix form takes modifiers after the type {#cast.9}.

## Integers, floats, bool

- A narrowing cast, or any sign change, is checked at runtime. `cast,trunc` and `cast,no_check` keep the low bits instead: `cast,trunc(u8) w` with `w := 300` is `44` {#cast.2}. Unchecked, a value is sign- or zero-extended and reinterpreted: `cast,no_check(u64)` of `s8 -1` is `u64` max {#cast.4}.
- A target that holds every source value (`u8` to `s16`, `s32` to `s64`) is not checked. Negative to any unsigned fails; unsigned above the signed max to a signed type fails {#cast.32}. Code that wants the bits (hashing) writes `no_check`.
- Float to integer truncates toward zero: `cast(s64) -3.99` is `-3` {#cast.3}. It saturates at the target range, and `NaN` is `0`.
- Integer to float rounds once, to nearest even.
- `cast,force` between a same-size integer and float reinterprets the bits {#cast.5}.
- A number, enum or pointer cast to `bool` compares with zero {#cast.7}; the result is exactly `true` or `false` (`!cast(bool) 2` is false) {#cast.22}. `cast(bool) ""` is false {#cast.6}.

## Pointers, strings, enums

- Pointers and integers convert both ways (`cast(*u8) 0 == null`) {#cast.23}; integer constants cast to pointers fold to constants {#cast.24}.
- `pointer & int`, `|` and `^` keep the pointer type, so allocator masks like `cast(u64) p & MASK` work {#cast.21}.
- `*void` converts implicitly to and from any pointer (`p: *T = alloc(size_of(T));`) {#cast.8}.
- `E.loose` and `E` convert to each other implicitly {#cast.15}.
- A string literal converts to `*u8`; a `string` variable does not {#cast.25}. Literals also convert to `#type,distinct` and `#type,isa` string variants {#cast.26}, and `.[...]` literals take an expected distinct array type of the same shape {#cast.27}.

## Bounds checks

`Build_Options.cast_bounds_check` is `.FATAL` (default), `.NONFATAL` or `.OFF`. It applies to workspaces a metaprogram creates, like the other checks.

```
cast.jai:3:5: error: runtime error: cast of 300 to `u8` overflows
help: `u8` holds 0 to 255; write `cast,trunc(u8)` (or `xx,trunc`) to keep the low bits, or `cast,no_check(u8)` to skip the check
```

- `.NONFATAL` warns on stderr and keeps the low bits; `.OFF` keeps them silently {#cast.33}.
- `xx` is checked like `cast`. `trunc`, `no_check`, `force`, enums and pointers skip the check {#cast.34}.
- Constants are checked at compile time: `cast(u8) 300` and `cast(u8) -1` are errors, `cast,trunc(u8) 300` is `44` {#cast.35}. A hex, binary, `~` or named constant may fill a signed type's bits (`cast(s8) 0xff` is `-1`; `cast(s8) 255` is not).

## `xx` and types from context

A value with no type of its own (`.NAME`, `.{...}`, `.[...]`, bare `xx`, an untyped number, `null`, or an `ifx` or arithmetic built from those) takes its type from a target (declaration, parameter, return, cast) or a sibling (other `ifx` arm, other operand, a candidate overload's parameter).

- `xx a == b` casts `a` to `b`'s type {#cast.11}.
- As a call argument, `xx a | b` (also `&`, `^`) takes the parameter's type and the other operand follows {#cast.12}. An untyped struct literal in arithmetic does the same: `f(.{300, -1} * scale)` builds the parameter's `Vector2` and uses its `operator *` {#cast.10}.
- The type the whole expression converts to is not a sibling: `target = xx (xx a + n);` adds in `n`'s type, then the outer `xx` converts.
- A float target does not reach operands of `&`, `|`, `^`, `%`: `f: float32 = (ifx c then a else b) & mask;` with `s16` arms computes in `s16` {#cast.31}.
- Between overloads, a bare `xx` argument prefers a scalar parameter to an array or slice one {#cast.37}.

## A cast does not type its operand

`cast(T) expr` types `expr` bottom-up, then converts {#cast.29}. With `w: u16 = 15`, `cast(float32) (0 - w)` subtracts in `u16` and converts `65521`. `cast(s32) (x + 100)` with `x: s8 = 100` wraps to `-56` before widening.

Only untyped operands take the cast's type: a bare `.NAME`, untyped `.{}`/`.[]`, and operators over those (`cast(Flags) (.A | .B)`). A bare number stays an untyped constant folded at the target: `cast(float32) (1 / 3)` is `0` {#cast.30}.

## `ifx` typing

- Arms must agree on a type, even for a constant `c`: `ifx c then 1 else "a"` is an error. An arm that needs an expected type (`.B`, `.{}`) takes the other arm's type.
- Two untyped integer constants take an unsigned target when both fit (`x: u32 = ifx c then 1 else 0;`); one that doesn't is `constant 300 does not fit in u8`.
- With no expected type, `ifx` takes the else type when only the then value converts to it, otherwise the then type {#cast.13}. Constant arms are an `s64` unless an unsigned or float type is wanted, so `return ifx c then 1 else 2;` from an `s32` procedure is an `implicit-narrowing` error; write `cast(s32) ifx ..` {#cast.36}.
- A call whose single result has exactly the base type of an `#type,isa` argument returns the variant {#cast.28}.

## Reach of a prefix cast

A prefix `cast(T)` or `xx` takes postfix and bitwise/shift operators into its operand; looser operators apply to the result.

| After the operand | Applies to | Example |
| --- | --- | --- |
| `.` `[]` `()` `.*` | operand | `cast(float) p.x / 2` is `(cast(float) p.x) / 2` {#cast.16} |
| `&` `\|` `^` `<<` `>>` `<<<` `>>>` | operand | `cast(float) (hex >> 16) & 0xFF` casts the masked value {#cast.17} |
| `+ - * / %` | result | `cast(*u8) p + 1` is `(cast(*u8) p) + 1` {#cast.18} |
| comparisons, `&&` `\|\|` | result | `cast(u8) n < 300` compares a `u8` {#cast.19} |

So `cast(u32) b << 4 | 1` is `cast(u32) ((b << 4) | 1)` {#cast.20}; write `(cast(u32) b) << 16` to widen first. A comma after a cast keyword is a modifier, so `(-cast,no_check(int) x)` is one expression {#cast.14}.

## How to change it

- Conversions live in `sema/convert.rs` (`explicit_cast`, `implicit_cost`, `emit_cast_check`); implicit cost also decides overload winners. Constants fold at the top of `explicit_cast`, so change that and the runtime path together.
- Cast reach is `CAST_PREC` in the parser. Moving it needs `tests/compiler/regressions/cast-operand-precedence.jai` updated.
- Widening `cast_operand_needs_target` in `sema/expr.rs` brings back top-down typing of operands.
