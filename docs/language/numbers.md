# Integers, floats, literals and overflow checks

The scalar types are `s8`..`s64`, `u8`..`u64`, `float32`/`float`, `float64` and `bool`; `int` is `s64` {#num.1}. Literals are untyped until a declaration, argument or operator types them; an untyped integer defaults to `s64` {#num.2}.

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

## Literals

- A decimal float is `float64` when it has eight or more counted fractional digits (whole digits, trailing zeros and, below one, leading zeros don't count: `0.000000001` is one digit), or a binary exponent outside -126..127. Otherwise it is `float32` {#num.3}. An untyped float prefers a `float32` overload {#num.4}.
- A float magnitude beyond `float32` can't be stored implicitly: `x: float32 = 1.0e39;` is an error, while `cast(float32) 1.0e39` is infinity {#num.18}. A decimal literal that becomes a different `float32` value (`16777217.0` is `16777216`) warns with the value it becomes.
- An integer literal must fit 64 bits {#num.19}. `18446744073709551615` is a valid `u64` {#num.9} and `-9223372036854775808` a valid `s64` {#num.10}. `1_000`, `0xFF_FF` and `0b1010` work {#num.11}.
- `0h` bit-pattern literals take exactly 4, 8 or 16 hex digits: 4 or 8 select `float32`, 16 `float64`.

## Conversion and arithmetic

- Implicit conversion only goes to a type holding the whole source range {#num.5}. Narrowing {#num.6} and constants that don't fit {#num.7} are errors.
- A negative constant never fits an unsigned type (`x: u32 = -1;`), and a decimal constant must fit the signed range (`x: s16 = 40000;`). A hex, binary, `~` or named constant may fill a signed type's bits: `x: s8 = 0xff;` is `-1` {#num.20}. An expression of decimal literals only (`100 + 100`) counts as decimal. `~0` fills a `u64`.
- Division and remainder truncate toward zero: `-7/2` is `-3`, `-7 % 3` is `-1` {#num.8}.
- A constant division by zero is a compile error, for floats too; a zero variable divisor gives `inf` or `-inf` at run time {#num.12}.
- Integers wrap at their width (`v: u8 = 255; v += 1` gives `0`) {#num.13} unless overflow checks are on {#num.14}.
- 128-bit integers are a library feature, `Basic/Int128.jai` {#num.15}.

## How literals meet other operands

Typing flows up the tree. An untyped literal takes the other operand's type.

- With `w: u16`, `0 - w` is `u16` arithmetic and wraps to `65521`, even assigned to a `float32`: `f: float32 = 0 - w;` subtracts in `u16`, then converts {#num.16}.
- A literal that doesn't fit the other operand makes the operation `s64`: with `a: u8`, `a + -1`, `a == -1` and `a + 256` are `s64`, and `a == -1` is `false` for `a = 255` {#num.21}.
- A hex or binary literal that fits as a bit pattern keeps the operand's type (`x & 0xffff_fff8` with a `u32` is `u32`).
- The complement of a constant (`~0x7`, `~0`) is a 64-bit pattern that doesn't adapt: next to a `u8`, `u16`, `u32` or `s32` it makes the operation `s64`, so `y: u32 = x & ~0x7;` is an error. Write `~cast(u32) 7`. Next to a `u64` it is the `u64` pattern. `y: u32 = ~0;` still fills the declared type.
- Two typed integer constants meet in the wider type: `cast(s16) 1 + cast(s32) 2` is `s32`.
- A `float32` operand meeting a float literal that needs `float64` (`x + 0.1591549431`) widens the operation to `float64`; only a declaration, argument or return taking it as `float32` is an error {#num.22}.
- A cast is none of these: `cast(T) expr` types `expr` on its own ([casts](casts-and-conversions.md)) {#num.17}.

## Overflow checks

`Build_Options.arithmetic_overflow_check` is `.OFF` (default), `.NONFATAL` or `.FATAL`. It makes integer `+`, `-`, `*` and their compound forms report results that don't fit {#overflow.1}.

```
file.jai:12: arithmetic overflow computing 250 + 10 as u8: the result does not fit
```

- `__arithmetic_overflow` prints to stderr {#overflow.3}. Fatal mode panics {#overflow.4}; `.NONFATAL` continues with the wrapped value {#overflow.5}.
- `#no_aoc` turns it off for code that overflows on purpose, like hashes and generators {#overflow.2}. It goes on a procedure, a block `#no_aoc { }`, or `while`, `if`, `for` {#overflow.6}.
- The option applies to workspaces a metaprogram creates; the program's own workspace keeps the default {#overflow.7}.
- Not checked: constant folding (wraps), unary minus, shifts, pointer arithmetic, floats, compiler-emitted indexing and `#asm` {#overflow.8}.

## How to change it

- Folding wraps with `int_arith::wrap` ([integer arithmetic](../compiler/integer-arithmetic.md)); conversions are `implicit_cost` and `scalar_convert` in `sema/convert.rs`; literal meeting is `literal_meets` in `sema/binary.rs`.
- Untyped constants carry `untyped: true` on `Operand::Const` until `settle_untyped`. Keep that flag in new folding paths or a literal silently becomes `s64`.
- A new path that lowers `+ - *` must call `emit_overflow_check` and honour `no_aoc`. Stdlib code that overflows on purpose needs `#no_aoc` (`Hash`, `PCG`, `Crc`, `xxHash`, `Random`, `Runtime_Support` have it).
