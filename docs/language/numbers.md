# Integers, floats and numeric literals

## What it is

The scalar number types: `s8`..`s64`, `u8`..`u64`, `float32`/`float`, `float64`, and `bool`. `int` is `s64` {#num.1}.

## How it works

Literals are untyped constants until a declaration, argument or operator gives them a type. An untyped integer defaults to `s64` {#num.2}. An untyped float is `float32` unless it has more than 7 significant figures and `float32` can't hold it exactly (`float_literal_type` in `sema/expr.rs`) {#num.3}. A `0h` bit-pattern literal is `float32` with up to 8 hex digits and `float64` with more (`0h7FEFFFFF_FFFFFFFF`), whatever its value; either converts to an expected float type.

```jai
a := 5;                    // s64
b := 2.25193;              // float32
c := 12342345234.0;        // float64: too many significant digits
u: u8 = 200;
w: u16 = u;                // widening is implicit
```

An untyped float literal also prefers a `float32` overload over a `float64` one {#num.4}.

Implicit conversion only goes to a type that holds the whole source range {#num.5}. Narrowing {#num.6}, and constants that don't fit {#num.7}, are errors:

```
error: type mismatch: expected `u8`, found `u16`        // b: u8 = some_u16;
error: constant 300 does not fit in u8              // c: u8 = 300;
```

- Division and remainder truncate toward zero {#num.8}: `-7/2` is `-3`, `-7 % 3` is `-1`.
- `18446744073709551615` is a valid `u64` literal {#num.9} and `-9223372036854775808` a valid `s64` one {#num.10}. `1_000`, `0xFF_FF` and `0b1010` work {#num.11}.
- Float division by zero gives `inf` or `-inf` {#num.12}.
- Integer arithmetic wraps at the declared width (`v: u8 = 255; v += 1` gives `0`) {#num.13} unless overflow checks are on {#num.14}; see [arithmetic overflow checks](arithmetic-overflow-checks.md).
- 128-bit integers are a library feature {#num.15}, `stdlib/Basic/Int128.jai`.

Types flow up the expression tree. A typed value never takes its type from its context; only untyped things are matched downward, and only in these places:

- An untyped literal meets a known type directly: a declaration (`a: u8 = 10;`), an argument, a return, or the other operand of a binary operator. In `0 - w` with `w: u16`, the `0` is a `u16` and the subtraction is `u16` arithmetic, so it wraps to `65521` (or fails an [overflow check](arithmetic-overflow-checks.md) when those are on). That holds even when the whole expression has an expected type: `f: float32 = 0 - w;` subtracts in `u16` and converts `65521` {#num.16}.
- When both operands are untyped (`2 * 3`), the expression folds as a constant and is matched to the context.
- Inferred enum members are pushed back through operators once the enum type is known (`d = .WEST | .EAST;`), and `.{...}` / `.[...]` literals are matched against a concrete struct or array type.

A cast is none of these: `cast(T) expr` types `expr` on its own and converts the result ([casts](casts-and-conversions.md)) {#num.17}.

## How to change it

Folding lives in `sema/expr.rs` (`wrap_int`); conversions in `sema/convert.rs` (`implicit_cost`, `scalar_convert`). In `check_binary`, a left operand that `is_number_literal` is checked without the expected type, so it settles to the right operand's type rather than the context's; the right operand already takes the left one's type. Type widths are fixed in `crates/jaic/src/types.rs`.

Gotcha: untyped constants carry `untyped: true` on `Operand::Const` until `settle_untyped` picks a type. Preserve that flag in any new folding path, or a literal silently becomes `s64`.

Tests: `tests/stdlib/float-literal-precision.jai`, `literal-matches-other-operand.jai`, `literal-prefers-float32.jai`, `int128-arithmetic.jai`.

## Dependencies

`lexer.rs` for literal tokens, `sema/convert.rs` and `sema/expr.rs` for typing and folding, the interpreter and `crates/jaic-llvm` for execution.
