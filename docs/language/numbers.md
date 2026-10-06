# Integers, floats and numeric literals

## What it is

The scalar number types: `s8`..`s64`, `u8`..`u64`, `float32`/`float`, `float64`, and `bool`. `int` is `s64`.

## How it works

Literals are untyped constants until a declaration, argument or operator gives them a type. An untyped integer defaults to `s64`. An untyped float is `float32` unless it has more than 7 significant figures and `float32` can't hold it exactly (`float_literal_type` in `sema/expr.rs`); either converts to an expected float type.

```jai
a := 5;                    // s64
b := 2.25193;              // float32
c := 12342345234.0;        // float64: too many significant digits
u: u8 = 200;
w: u16 = u;                // widening is implicit
```

An untyped float literal also prefers a `float32` overload over a `float64` one.

Implicit conversion only goes to a type that holds the whole source range. Narrowing, and constants that don't fit, are errors:

```
error: type mismatch: expected u8, found u16        // b: u8 = some_u16;
error: constant 300 does not fit in u8              // c: u8 = 300;
```

- Division and remainder truncate toward zero: `-7/2` is `-3`, `-7 % 3` is `-1`.
- `18446744073709551615` is a valid `u64` literal and `-9223372036854775808` a valid `s64` one. `1_000`, `0xFF_FF` and `0b1010` work.
- Float division by zero gives `inf` or `-inf`.
- Integer arithmetic wraps at the declared width (`v: u8 = 255; v += 1` gives `0`) unless overflow checks are on; see [arithmetic overflow checks](arithmetic-overflow-checks.md).
- 128-bit integers are a library feature, `stdlib/Basic/Int128.jai`.

## How to change it

Folding lives in `sema/expr.rs` (`wrap_int`); conversions in `sema/convert.rs` (`implicit_cost`, `scalar_convert`). Type widths are fixed in `crates/jaic/src/types.rs`.

Gotcha: untyped constants carry `untyped: true` on `Operand::Const` until `settle_untyped` picks a type. Preserve that flag in any new folding path, or a literal silently becomes `s64`.

Tests: `tests/stdlib/float-literal-precision.jai`, `literal-prefers-float32.jai`, `int128-arithmetic.jai`.

## Dependencies

`lexer.rs` for literal tokens, `sema/convert.rs` and `sema/expr.rs` for typing and folding, the interpreter and `crates/jaic-llvm` for execution.
