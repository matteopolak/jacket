# Integers, floats, literals and overflow checks

## What it is

The scalar types `s8`..`s64`, `u8`..`u64`, `float32`/`float`, `float64`, `bool` (`int` is `s64`), numeric literals, implicit conversion, and the optional arithmetic overflow check.

## How it works

Literals are untyped until a declaration, argument or operator types them; an untyped integer defaults to `s64`. A decimal float is `float64` when it has eight or more counted fractional digits (whole digits and trailing zeros do not count) or a binary exponent outside -126..127; otherwise `float32`. An untyped float prefers a `float32` overload. `0h` literals take exactly 4, 8 or 16 hex digits.

```jai
a := 5;                    // s64
b := 2.25193;              // float32
c := 0.12345678;           // float64
u: u8 = 200;
w: u16 = u;                // widening is implicit
x: u8 = some_u16;          // error: type mismatch
y: u8 = 300;               // error: constant 300 does not fit in u8
z: s8 = 0xff;              // -1: hex, binary and ~ constants may fill the bits
```

- Implicit conversion only goes to a type holding the whole source range. A negative constant never fits unsigned; a decimal constant must fit the signed range. A constant that is an expression of decimal literals counts as decimal (`int_constant_fits`, `bit_pattern` in `sema/convert.rs`).
- A float magnitude beyond float32 cannot be stored implicitly (`x: float32 = 1.0e39;` is an error; `cast(float32)` gives infinity). Integer literals must fit 64 bits.
- Division and remainder truncate toward zero. Float division by zero gives `inf`. Integers wrap at their width. 128-bit integers are a library feature.
- Typing flows up the tree. An untyped literal takes the other operand's type: with `w: u16`, `0 - w` is `u16` arithmetic (wraps to 65521) even when assigned to a `float32`. A literal that does not fit the other operand makes the operation `s64` (`a + -1`, `a == 256` with `a: u8`). A hex/binary/`~` literal that fits as a bit pattern keeps the operand's type (`literal_meets`, `literal_as_bits` in `sema/expr.rs`). Two typed integer constants meet in the wider type (`fold_binary`). A cast types its operand on its own ([casts](casts-and-conversions.md)).

### Overflow checks

`Build_Options.arithmetic_overflow_check` (`.OFF` default, `.NONFATAL`, `.FATAL`) makes integer `+ - *` (and compound forms) report results that do not fit. `#no_aoc` disables it on a procedure, block, `while`, `if` or `for` (for hashes and generators; `ProcFlags::no_aoc`, `Block::no_aoc`).

```
file.jai:12: arithmetic overflow computing 250 + 10 as u8: the result does not fit
```

`check_binary` (`sema/expr.rs`) calls `emit_overflow_check`, which emits `Intrinsic::{S,U}{Add,Sub,Mul}Overflow` and a call to `__arithmetic_overflow` (`stdlib/internal/runtime/faults.jai`, re-exported by `Runtime_Support`). The interpreter uses 128-bit arithmetic, LLVM uses `llvm.*.with.overflow`. Fatal mode panics; non-fatal continues with the wrapped value. The option applies to workspaces a metaprogram creates, not the program's own. Not checked: constant folding, unary minus, shifts, pointer arithmetic, floats, compiler-emitted indexing and `#asm`.

## How to change it

Folding is `wrap_int` in `sema/expr.rs`; conversions are `implicit_cost` and `scalar_convert` in `sema/convert.rs`; widths are in `crates/jaic/src/types.rs`. Untyped constants carry `untyped: true` on `Operand::Const` until `settle_untyped`; keep that flag in new folding paths or a literal silently becomes `s64`. A new path that lowers `+ - *` must call `emit_overflow_check` and honour `no_aoc`. Stdlib code that overflows on purpose needs `#no_aoc` (`Hash`, `PCG`, `Crc`, `xxHash`, `Random`, `Runtime_Support` have it). Tests: `tests/stdlib/float-literal-precision.jai`, `literal-matches-other-operand.jai`, `arithmetic-overflow-check.jai`, `tests/corpus/negative/aoc-fatal-*.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `Build_Options.arithmetic_overflow_check` (`.OFF` by default, `.NONFATAL`, `.FATAL`) makes integer `+`, `-` and `*` (and `+=`, `-=`, `*=`) report results that do not fit the operand type {#overflow.1}. `#no_aoc` turns the check off for code that overflows on purpose, like hashes and random number generators {#overflow.2}.
- `__arithmetic_overflow` prints to stderr {#overflow.3}:
- Bit 15 of `type_code` marks fatal mode, which then panics {#overflow.4}; `.NONFATAL` returns and the program continues with the wrapped value {#overflow.5}.
- `#no_aoc` goes where `#no_abc` goes: on a procedure (`ProcFlags::no_aoc`, setting `FnCtx::no_aoc` in `sema/procs.rs`), on a block `#no_aoc { }`, and on `while`, `if` and `for` (`Block::no_aoc`, `sema/stmt.rs`) {#overflow.6}.
- The option reaches sema as `Options::arithmetic_overflow_check` (0, 1, 2) via the workspace option in `stdlib/Compiler/options.jai` and `build.rs`. It applies to workspaces a metaprogram creates; the program's own workspace keeps the default {#overflow.7}.
- Not checked: constant folding (wraps), unary minus, shifts, pointer arithmetic, floats, and arithmetic the compiler emits for indexing and `#asm` {#overflow.8}.
- The scalar number types: `s8`..`s64`, `u8`..`u64`, `float32`/`float`, `float64`, and `bool`. `int` is `s64` {#num.1}.
- Literals are untyped constants until a declaration, argument or operator gives them a type. An untyped integer defaults to `s64` {#num.2}. Decimal float default precision uses the reviewed public lexer heuristic: digits before the decimal are excluded, trailing fractional zeros are ignored, and eight or more counted fractional digits select `float64`. A normal binary exponent outside -126 through 127 also selects `float64`; zero, subnormal float64 values, infinity and NaN do not have a big normal exponent under this rule {#num.3}. The compiler exports the documented default/required precision flags with literal AST records. A magnitude outside float32's normal range cannot be stored implicitly in a `float32`: `x: float32 = 1.0e39;` is an error, while `cast(float32) 1.0e39` is infinity {#num.18}. An integer literal must fit in 64 bits {#num.19}. A `0h` bit-pattern literal requires exactly 4, 8 or 16 hex digits (underscores excluded); 4/8 digits select `float32` and 16 select `float64`.
- An untyped float literal also prefers a `float32` overload over a `float64` one {#num.4}.
- Implicit conversion only goes to a type that holds the whole source range {#num.5}. Narrowing {#num.6}, and constants that don't fit {#num.7}, are errors:
- A negative constant never fits an unsigned type (`x: u32 = -1;` is an error), and a decimal constant must fit a signed type's range (`x: s16 = 40000;` is an error). A hex, binary, `~` or named constant may fill a signed type's bits instead: `x: s8 = 0xff;` is `-1` {#num.20}. Every 64-bit literal fits `s64` (`9223372036854775808` is its minimum), and `~0` fills a `u64`. `int_constant_fits` and `bit_pattern` in `sema/convert.rs` decide; a constant that is an expression of decimal literals only (`100 + 100`) counts as decimal, any other expression as a possible bit pattern.
- Division and remainder truncate toward zero {#num.8}: `-7/2` is `-3`, `-7 % 3` is `-1`.
- `18446744073709551615` is a valid `u64` literal {#num.9} and `-9223372036854775808` a valid `s64` one {#num.10}. `1_000`, `0xFF_FF` and `0b1010` work {#num.11}.
- Float division by zero gives `inf` or `-inf` {#num.12}.
- Integer arithmetic wraps at the declared width (`v: u8 = 255; v += 1` gives `0`) {#num.13} unless overflow checks are on {#num.14}; see [arithmetic overflow checks](numbers.md).
- 128-bit integers are a library feature {#num.15}, `stdlib/Basic/Int128.jai`.
- An untyped literal meets a known type directly: a declaration (`a: u8 = 10;`), an argument, a return, or the other operand of a binary operator. In `0 - w` with `w: u16`, the `0` is a `u16` and the subtraction is `u16` arithmetic, so it wraps to `65521` (or fails an [overflow check](numbers.md) when those are on). That holds even when the whole expression has an expected type: `f: float32 = 0 - w;` subtracts in `u16` and converts `65521` {#num.16}. A literal that does not fit the other operand's type makes the operation `s64` instead: with `a: u8`, `a + -1`, `a == -1` and `a + 256` are `s64`, and `a == -1` is `false` for `a = 255` {#num.21}. A hex, binary or `~` literal that fits only as a bit pattern keeps the operand's type (`mask & ~0x7` with a `u32` mask stays `u32`; `literal_meets` and `literal_as_bits` in `sema/expr.rs`). Typed constants of two integer types meet in the wider one, as variables do: `cast(s16) 1 + cast(s32) 2` is `s32` (`fold_binary`).
- A cast is none of these: `cast(T) expr` types `expr` on its own and converts the result ([casts](casts-and-conversions.md)) {#num.17}.
