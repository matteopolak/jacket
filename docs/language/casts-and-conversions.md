# Casts and conversions

## What it is

How a value changes type: implicit conversion, `cast(T)`, the contextual `xx`, postfix `.(T)`, and the `no_check`, `trunc` and `force` modifiers.

## How it works

`explicit_cast` in `sema/convert.rs` handles every explicit form; `parse_cast_flags` in `parser/expr.rs` reads modifiers into `CastFlags { no_check, truncate, force }`.

```jai
small := cast,no_check(u8) wide;     // u16 298 -> 42
t := cast,trunc(u8) 256;             // 0
x: u8 = xx,trunc 298;                // 42, target from the declaration
p := (42).(u8);  q := big.(u32, trunc);   // postfix forms
bits := cast,force(u32) f;           // reinterprets float bits
```

- Narrowing integer casts are checked (see below); `trunc` and `no_check` keep the low bits. Casts to an equal or wider type are never checked (`cast(u64)` of `s8 -1` is `u64` max).
- Float to integer truncates toward zero. Integer to float rounds once, to nearest even, at the target width (no double rounding through `float64`); the interpreter, constant folding (`int_to_float`) and LLVM agree.
- A number, enum, pointer or string cast to `bool` compares with zero. Pointers and integers convert both ways. `pointer & int`, `|`, `^` keep the pointer type (allocator code masks pointers). `*void` converts implicitly to and from any pointer. `E.loose` and `E` convert implicitly. A string literal converts to `*u8`; a `string` variable does not.
- `xx` takes the type the context expects (declaration, parameter, return). In `xx a == b` it takes `b`'s type. As an argument, `xx a | b` takes the parameter's type and the other operand follows (`autocast_arithmetic`); an untyped struct literal in arithmetic does the same (`literal_arithmetic`).
- A cast does not push its type into its operand: `cast(T) expr` types `expr` bottom-up, then converts. With `w: u16`, `cast(float32) (0 - w)` subtracts in `u16`; `cast(s32) (x + 100)` with `x: s8 = 100` wraps in `s8` first. Only operands with no type of their own take the cast's type (bare `.NAME`, untyped `.{}`/`.[]`, operators over those), and a bare number stays an untyped constant folded at the target (`cast(float32) (1 / 3)` is `0`).
- The two branches of an `ifx` must agree on a type: `ifx c then 1 else "a"` is an error even for a constant `c` (the branch not taken is still checked, `fold_constant_ifx` and `ifx_branch_agreement` in `sema/expr.rs`), with a note naming each branch's type.
- `ifx c then a else b` with no expected type takes the else type when only the then value converts to it, otherwise the then type. A call whose single result has exactly the base type of an `#type,isa` argument returns the variant (`emit_call`).

### Reach of a prefix cast

A prefix `cast(T)` or `xx` sits at `CAST_PREC` ([operators](operators.md)): its operand takes postfix and bitwise/shift operators; looser operators apply to the result.

| After the operand | Applies to | Example |
| --- | --- | --- |
| `.` `[]` `()` `.*`, `&` `\|` `^` `<<` `>>` | operand | `cast(float) (h >> 16) & 0xFF` casts the masked value |
| `+ - * / %`, comparisons, `&&` `\|\|` | result | `cast(*u8) p + 1` is `(cast(*u8) p) + 1` |

So `cast(u32) b << 4 | 1` is `cast(u32) ((b << 4) | 1)`; write `(cast(u32) b) << 16` to widen first. This grouping is the one existing programs need (`cast(bool) flags & .REVERSE`, base64 encoders shifting before widening). A comma after a cast keyword is a modifier, so `(-cast,no_check(int) x)` is one expression.

### Cast bounds checks

`Build_Options.cast_bounds_check` (`.FATAL` default, `.NONFATAL`, `.OFF`) checks runtime integer casts to a narrower type. `.NONFATAL` warns and keeps the low bits; `.OFF` keeps them silently; the option applies to metaprogram-created workspaces.

```
cast.jai:3:5: error: runtime error: cast of 300 to `u8` overflows
help: `u8` holds 0 to 255; write `cast,trunc(u8)` (or `xx,trunc`) to keep the low bits, or `cast,no_check(u8)` to skip the check
```

`xx` is checked like `cast`; `trunc`, `no_check`, `force`, enums and pointers skip it. Constants are checked at compile time (`check_constant_cast`): `cast(u8) 300` and `cast(u8) -1` are errors, `cast,trunc(u8) 300` is `44`; a hex, binary, `~` or named constant may fill a signed type's bits (`cast(s8) 0xff` is `-1`).

## How to change it

- Cast reach: `parse_cast_value` in `parser/expr.rs` calls `parse_binary_after(first, CAST_PREC)`; move `CAST_PREC` or `binary_op` entries, and update `tests/compiler/regressions/cast-operand-precedence.jai` and the parser test `prefix_cast_takes_bitwise_and_shift_operators`.
- Scalar conversions: `scalar_convert`; implicit ones are priced by `implicit_cost`, which also decides overload winners. Constants fold at the top of `explicit_cast`; change that and the runtime path together.
- Operand typing: the `E::Cast` arm of `check_expr` (`sema/expr.rs`) passes the target down only when `cast_operand_needs_target` says so; widening it brings back top-down typing.
- Bounds check: `emit_cast_check` (`sema/convert.rs`) branches to `Intrinsic::CheckFailed` (`ir::TRAP_CAST_OVERFLOW`); native code calls `runtime_support_check_failed`; messages are `ir::check_message` and `cast_help` (`sema/trap_report.rs`).

Tests: `tests/compiler/regressions/lang-conversions.jai`, `cast-operand-typed-bottom-up.jai`, `cast-bounds-check.jai`, `int-to-float32-rounding.jai`, `tests/corpus/negative/cast-check-*.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- The example produces the values in its comments {#cast.1}; the postfix form takes modifiers after the type {#cast.9}.
- A cast to a narrower integer type checks that the value fits (see [cast bounds checks](#cast-bounds-checks)); `cast,trunc` and `cast,no_check` keep the low bits instead: `cast,trunc(u8) w` with `w := 300` is `44` {#cast.2}. A cast to a type at least as wide is never checked: `cast(u64) n` with `n: s8 = -1` is `18446744073709551615` {#cast.4}.
- Float-to-integer truncates toward zero: `cast(s64) -3.99` is `-3` {#cast.3}.
- `cast,force` between a same-size integer and float reinterprets the bits {#cast.5}.
- A number, enum or pointer cast to `bool` compares with zero {#cast.7}, so the result is exactly `true` or `false` (`!cast(bool) 2` is false) {#cast.22}. `cast(bool) ""` is false {#cast.6}.
- Pointers and integers convert both ways (`cast(s64) ptr`, `cast(*u8) addr`, `cast(*u8) 0 == null`) {#cast.23}. Integer constants cast to pointers fold to constants {#cast.24}.
- `pointer & int`, `pointer | int` and `pointer ^ int` are legal and keep the pointer type, because allocator code masks pointers with `cast(u64) p & MASK` {#cast.21}.
- `*void` accepts any pointer implicitly, and converts implicitly back to any pointer type (`p: *T = alloc(size_of(T));`) {#cast.8}.
- `E.loose` and `E` convert to each other implicitly {#cast.15}.
- A string literal converts to `*u8`; a `string` variable does not (see [strings and literals](strings-and-literals.md)) {#cast.25}. String literals also convert to `#type,distinct` and `#type,isa` string variants {#cast.26}, and `.[...]` literals take an expected distinct array type of the same shape {#cast.27}.
- In a comparison, `xx a == b` casts `a` to `b`'s type; `check_binary` checks the other side first {#cast.11}.
- As a call argument, `xx a | b` (also `&`, `^`) takes the parameter's type and the other operand follows it (`autocast_arithmetic` in `sema/calls.rs`) {#cast.12}. Arithmetic on an untyped struct literal works the same way: `f(.{300, -1} * scale)` builds the parameter's `Vector2` and then uses its `operator *` (`literal_arithmetic`) {#cast.10}.
- A cast does not push its type into its operand. `cast(T) expr` types `expr` bottom-up, exactly as it would be typed on its own, and then converts it. So with `w: u16 = 15`, `cast(float32) (0 - w)` subtracts in `u16` (the `0` is matched with `w`, see [numbers](numbers.md)) and converts `65521`, and `cast(float32) ((0 - w) & v)` is `u16` arithmetic. `cast(s32) (x + 100)` with `x: s8 = 100` adds in `s8` and wraps to `-56` before widening {#cast.29}.
- Only operands with no type of their own take the cast's type: a bare `.NAME`, an untyped `.{...}` or `.[...]`, and operators or an `ifx` built only from those (`cast(Flags) (.A | .B)`). A bare number (`cast(float32) 3`, `cast(u8) 255`) or a literal-only expression stays an untyped constant and is folded at the target: `cast(float32) (1 / 3)` is `0`, integer division first {#cast.30}.
- An expected float type from a declaration or a parameter does not reach the operands of `&`, `|`, `^` and `%`, which floats lack: `f: float32 = (ifx c then a else b) & mask;` with `s16` branches computes in `s16` and converts the result {#cast.31}.
- `ifx c then a else b` with no expected type takes the else type when only the then value converts to it (`ifx c then 0 else some_float` is a float); otherwise the then type wins {#cast.13}.
- A call whose single result has exactly the base type of an `#type,isa` argument returns the variant (`pa + pb` on `Position3` stays `Position3`; `emit_call` in `sema/calls.rs`) {#cast.28}.
- | `.`, `[]`, `()`, `.*` | operand | `cast(float) p.x / 2` is `(cast(float) p.x) / 2` {#cast.16} |
- | `&`, `\|`, `^`, `<<`, `>>`, `<<<`, `>>>` | operand | `cast(float) (hex >> 16) & 0xFF` is `cast(float) ((hex >> 16) & 0xFF)` {#cast.17} |
- | `+`, `-`, `*`, `/`, `%` | result | `cast(*u8) p + 1` is `(cast(*u8) p) + 1` {#cast.18} |
- | comparisons, `&&`, `\|\|` | result | `cast(u8) n < 300` compares a `u8` {#cast.19} |
- So `cast(u32) b << 4 | 1` is `cast(u32) ((b << 4) | 1)` {#cast.20}; write `(cast(u32) b) << 16` to widen before shifting.
- A comma after a cast keyword is a modifier, not a list separator, so `(-cast,no_check(int) x)` parses as one expression {#cast.14}.
- A negative value does not fit an unsigned target (`cast(u16)` of `n: s32 = -1` fails). Casts to a type of the same size or wider only extend or reinterpret the bits, so `cast(s64)` of a large `u64` and `cast(u32)` of a negative `s32` are never checked {#cast.32}. Evidence: `26.22_insert_scope.jai` in The Way to Jai prints negative numbers from `cast(int) random_get() % 100`, where `random_get` returns a `u64`.
- `.NONFATAL` prints `path:line: warning: cast of 300 to `u8` overflows` on stderr and goes on with the low bits; `.OFF` keeps the low bits silently. The option applies to workspaces a metaprogram creates, like the other checks {#cast.33}.
- `xx` to a narrower type is checked like `cast`; `trunc`, `no_check` and `force` skip the check, and so do enums and pointers {#cast.34}.
- A constant is checked at compile time instead (`check_constant_cast` in `sema/convert.rs`): `cast(u8) 300`, `cast(u8) -1` and `y: u8 = xx -1` are errors, and `cast,trunc(u8) 300` is `44` {#cast.35}. A hex, binary, `~` or named constant may fill a signed type's bits, so `cast(s8) 0xff` is `-1` and `cast(s32) 0xFFFF_FFFF` is accepted; `cast(s8) 255` is not.
