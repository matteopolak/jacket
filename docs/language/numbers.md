# Integers, floats and numeric literals

## What it is

The scalar number types of Jai as `jaic` implements them: `s8`..`s64`, `u8`..`u64`, `float32`/`float` and `float64`, plus `bool`. `int` is `s64`.

## How it works

Literals are untyped constants until a declaration, argument or operator gives them a type. An untyped integer defaults to `s64`; an untyped float literal is `float32` unless it needs more precision than `float32` holds.

```jai
a := 5;                    // s64
b := 2.25193;              // float32
c := 12342345234.0;        // float64 (too many significant digits)
u: u8 = 200;
w: u16 = u;                // widening is implicit
```

`tests/stdlib/float-literal-precision.jai` pins the float rule, and `tests/stdlib/literal-prefers-float32.jai` shows that an untyped literal picks a `float32` overload over a `float64` one.

Implicit conversion only goes to a type that holds the whole source range. Narrowing is a compile error, as is a constant that does not fit:

```
error: type mismatch: expected u8, found u16        // b: u8 = some_u16;
error: constant 300 does not fit in u8              // c: u8 = 300;
```

Division and remainder truncate toward zero (`-7/2` is `-3`, `-7 % 3` is `-1`). `u64` accepts literals up to `18446744073709551615`, and `-9223372036854775808` is a valid `s64` literal. `1_000`, `0xFF_FF` and `0b1010` literals are accepted. Float division by zero yields `inf` / `-inf`.

Integer arithmetic wraps at the declared width: `v: u8 = 255; v += 1` gives `0`. By default nothing is checked. A metaprogram can turn on overflow checks for `+`, `-` and `*` with `Build_Options.arithmetic_overflow_check` (`.NONFATAL` / `.FATAL`), and `#no_aoc` opts code out again; see [arithmetic-overflow-checks.md](arithmetic-overflow-checks.md).

128-bit arithmetic is a library feature: `stdlib/Basic/Int128.jai`, exercised by `tests/stdlib/int128-arithmetic.jai`.

## How to change it

Constant folding of numeric operations lives in `crates/jaic/src/sema/expr.rs` (`wrap_int`) and conversions in `crates/jaic/src/sema/convert.rs` (`implicit_cost`, `scalar_convert`). Untyped constants carry `untyped: true` on `Operand::Const` until `settle_untyped` fixes a type; keep that flag when adding folding paths or a literal silently becomes `s64`.

Overflow checking applies to runtime arithmetic only (constant folding still wraps, and literals that do not fit are errors); see [arithmetic-overflow-checks.md](arithmetic-overflow-checks.md).

## Configuration

None. Type widths are fixed in `crates/jaic/src/types.rs`.

## Dependencies

`crates/jaic/src/lexer.rs` for literal tokens, `sema/convert.rs` and `sema/expr.rs` for typing and folding, the interpreter and `crates/jaic-llvm` for execution.
