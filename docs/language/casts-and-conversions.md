# Casts and conversions

## What it is

How a value changes type: implicit conversion, `cast(T)`, the contextual `xx`, postfix `.(T)`, and the `no_check`, `trunc` and `force` modifiers.

## How it works

`explicit_cast` in `crates/jaic/src/sema/convert.rs` handles every explicit form. Modifiers are parsed by `parse_cast_flags` in `crates/jaic/src/parser/expr.rs` into `CastFlags { no_check, truncate, force }` (`trunc` and `truncate` are synonyms).

```jai
small := cast,no_check(u8) wide;     // u16 298 -> 42
t := cast,trunc(u8) 256;             // 0
x: u8 = xx,trunc 298;                // 42, target comes from the declaration
p := (298).(u8);                     // postfix form, 42
bits := cast,force(u32) f;           // float 3.7 -> 1080872141, reinterprets bits
```

Observed behavior (checked with `jaic run`):

- Integer-to-integer casts wrap to the target width, with or without a modifier. `cast(u8) w` with `w := 300` gives `44`; there is no range trap.
- Float-to-integer casts truncate toward zero: `cast(s64) -3.99` is `-3`.
- `cast(u64) n` for `n: s8 = -1` gives `18446744073709551615`.
- `cast,force` between a same-size integer and float reinterprets the bits.
- `cast(bool) 5` is true, `cast(bool) ""` is false.
- `*void` accepts any pointer implicitly; going back needs `cast(*T)` or `xx`.
- The postfix form takes modifiers after the type: `big.(u32, trunc)` (AST_Utils hashes `k.(*Type_Info).(u32,trunc)`).
- In a comparison, `xx a == b` casts `a` to `b`'s type (`xx err == GL_FALSE` with `err: s32`, `GL_FALSE` a bool): the other side is checked first (`check_binary`).
- As a call argument, `xx a | b` (also `&`, `^`, and inside a macro) takes the parameter's type, and the other operand follows it (`add_piece(p, xx a1|h1)` with `enum_flags` squares): `autocast_arithmetic` defers the whole binary to the parameter (`tests/stdlib/autocast-bitwise-argument.jai`).
- `ifx c then a else b` without an expected type takes the else-type when only the then-value converts implicitly to it (`ifx c then 0 else some_float` is a float). The then-type still wins otherwise (`tests/stdlib/ifx-widens-to-else.jai`).
- `(-cast,no_check(int) x)` in parentheses is an expression; the comma after a cast keyword is a modifier, not a list separator (`tests/stdlib/cast-modifier-in-parens.jai`).
- `E.loose` and `E` convert to each other implicitly (a node's `operator_type: Operator_Type.loose` passed as `Operator_Type`).

Pointers and integers convert both ways: `cast(s64) ptr`, `cast(*u8) addr`, and `cast(*u8) 0 == null`. Integer constants cast to pointers fold to constants (`tests/stdlib/const-integer-pointer.jai`). Enum conversions are covered by `tests/stdlib/lang-conversions.jai`.

A string literal converts to `*u8`; a `string` variable does not (see [strings-and-literals.md](strings-and-literals.md)).

- String literals convert implicitly to `#type,distinct` / `#type,isa` string variants (call matching returns `LITERAL` cost; `convert_const` retags the constant), and `.[...]` literals take an expected distinct array type of the same shape.
- A call to a procedure with an `#type,isa` variant argument whose single non-macro result has exactly the variant's base type returns the variant (`pa + pb` on `Position3` stays `Position3`); see `emit_call` in `sema/calls.rs`.

## How to change it

Add a scalar conversion in `scalar_convert`, or a new aggregate/array case in the later branches of `explicit_cast`. Implicit conversions are priced by `implicit_cost`; overload resolution uses that cost, so changing it changes which overload wins. Constant operands are folded at the top of `explicit_cast`; update that table together with the runtime path or `#run` results and runtime results will diverge.

## Configuration

None.

## Dependencies

`sema/convert.rs`, `sema/expr.rs` (`truthy`, `wrap_int`), the parser cast forms, and `ConvOp` in `crates/jaic/src/ir.rs`.
