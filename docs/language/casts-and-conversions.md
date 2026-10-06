# Casts and conversions

## What it is

How a value changes type: implicit conversion, `cast(T)`, the contextual `xx`, postfix `.(T)`, and the `no_check`, `trunc` and `force` modifiers.

## How it works

`explicit_cast` in `crates/jaic/src/sema/convert.rs` handles every explicit form. `parse_cast_flags` in `parser/expr.rs` reads the modifiers into `CastFlags { no_check, truncate, force }` (`trunc` and `truncate` are synonyms).

```jai
small := cast,no_check(u8) wide;     // u16 298 -> 42
t := cast,trunc(u8) 256;             // 0
x: u8 = xx,trunc 298;                // 42, target comes from the declaration
p := (298).(u8);                     // postfix form, 42
q := big.(u32, trunc);               // postfix with a modifier
bits := cast,force(u32) f;           // float 3.7 -> 1080872141, reinterprets bits
```

Scalar rules:

- Integer-to-integer casts wrap to the target width, with or without a modifier. `cast(u8) w` with `w := 300` is `44`; there is no range trap. `cast(u64) n` with `n: s8 = -1` is `18446744073709551615`.
- Float-to-integer truncates toward zero: `cast(s64) -3.99` is `-3`.
- Integer-to-float casts round once, to nearest even, at the target width. `cast(float32)` of a 64-bit integer does not go through `float64`, which would round twice: `2^63 + 2^39 + 1` must give the `float32` above the tie. The interpreter (`conv` in `interp/mod.rs`), constant folding (`int_to_float` in `sema/convert.rs`) and LLVM (`sitofp`/`uitofp`) agree (`tests/stdlib/int-to-float32-rounding.jai`).
- `cast,force` between a same-size integer and float reinterprets the bits.
- A number, enum or pointer cast to `bool` compares with zero, so the result is exactly `true` or `false` (`!cast(bool) 2` is false). `cast(bool) ""` is false.
- Pointers and integers convert both ways (`cast(s64) ptr`, `cast(*u8) addr`, `cast(*u8) 0 == null`). Integer constants cast to pointers fold to constants.
- `pointer & int`, `pointer | int` and `pointer ^ int` are legal and keep the pointer type, because allocator code masks pointers with `cast(u64) p & MASK`.
- `*void` accepts any pointer implicitly; going back needs `cast(*T)` or `xx`.
- `E.loose` and `E` convert to each other implicitly.
- A string literal converts to `*u8`; a `string` variable does not (see [strings and literals](strings-and-literals.md)). String literals also convert to `#type,distinct` and `#type,isa` string variants, and `.[...]` literals take an expected distinct array type of the same shape.

Where the target type comes from:

- `xx` takes the type the context expects: a declaration, a parameter, a return.
- In a comparison, `xx a == b` casts `a` to `b`'s type; `check_binary` checks the other side first.
- As a call argument, `xx a | b` (also `&`, `^`) takes the parameter's type and the other operand follows it (`autocast_arithmetic` in `sema/calls.rs`). Arithmetic on an untyped struct literal works the same way: `f(.{300, -1} * scale)` builds the parameter's `Vector2` and then uses its `operator *` (`literal_arithmetic`).
- `ifx c then a else b` with no expected type takes the else type when only the then value converts to it (`ifx c then 0 else some_float` is a float); otherwise the then type wins.
- A call whose single result has exactly the base type of an `#type,isa` argument returns the variant (`pa + pb` on `Position3` stays `Position3`; `emit_call` in `sema/calls.rs`).

### How far a prefix cast reaches

A prefix `cast(T)` or `xx` is a unary operator at `CAST_PREC`, between the bitwise operators and `*` (see [operators](operators.md)). Its operand takes postfix and bitwise/shift operators; everything looser applies to the cast's result:

| After the cast operand | Applies to | Example |
| --- | --- | --- |
| `.`, `[]`, `()`, `.*` | operand | `cast(float) p.x / 2` is `(cast(float) p.x) / 2` |
| `&`, `\|`, `^`, `<<`, `>>`, `<<<`, `>>>` | operand | `cast(float) (hex >> 16) & 0xFF` is `cast(float) ((hex >> 16) & 0xFF)` |
| `+`, `-`, `*`, `/`, `%` | result | `cast(*u8) p + 1` is `(cast(*u8) p) + 1` |
| comparisons, `&&`, `\|\|` | result | `cast(u8) n < 300` compares a `u8` |

So `cast(u32) b << 4 | 1` is `cast(u32) ((b << 4) | 1)`; write `(cast(u32) b) << 16` to widen before shifting.

Why: real code only type-checks or matches its recorded output with this grouping. `cast(bool) flags & .REVERSE` and float `cast(float) (hex >> 16) & 0xFF` are errors if `&` applies to the result. open-jai's `utils/stress.jai` base64 encoder builds `(cast(u32) a << 16) | ...` from `u8`s, and its recorded output only matches if the shift happens before widening. Pointer differences like `cast(s64) p2 - cast(s64) p0 == 8` and `cast(float) width * v` (with `v` a `Vector2`) need `-` and `*` to stop. The the changelog also says a cast of a sum needs parentheses.

A comma after a cast keyword is a modifier, not a list separator, so `(-cast,no_check(int) x)` parses as one expression.

## How to change it

- **Cast reach:** `parse_cast_value` in `parser/expr.rs` parses a unary operand, then calls `parse_binary_after(first, CAST_PREC)`. Move `CAST_PREC` or operators in `binary_op` to change what a cast absorbs, and update `tests/stdlib/cast-operand-precedence.jai` and the parser test `prefix_cast_takes_bitwise_and_shift_operators`.
- **Scalar conversions:** `scalar_convert`; aggregate and array cases are later branches of `explicit_cast`.
- **Implicit conversions:** priced by `implicit_cost`. Overload resolution uses that cost, so changing it changes which overload wins.
- **Constants:** folded at the top of `explicit_cast`. Change that and the runtime path together, or `#run` and runtime results diverge.

Tests: `tests/stdlib/cast-to-bool-nonzero.jai`, `lang-conversions.jai`, `const-integer-pointer.jai`, `literal-arithmetic-argument.jai`, `autocast-bitwise-argument.jai`, `ifx-widens-to-else.jai`, `cast-modifier-in-parens.jai`.

## Dependencies

`parser/expr.rs`, `sema/convert.rs`, `sema/calls.rs`, `sema/expr.rs` (`truthy`, `wrap_int`) and `ConvOp` in `crates/jaic/src/ir.rs`.
