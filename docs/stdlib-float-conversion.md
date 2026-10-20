# Float conversion in Basic

`string_to_float64`, `string_to_float` and the float cases of `print`/`formatFloat` convert between
binary floats and decimal text. Every result is exact: parsing returns the correctly rounded float
(ties to even), and formatting prints the correctly rounded digits of the exact binary value. Each
direction has fast paths for common inputs and one big-integer path that handles everything.

## Parsing

`parse_float_number` (`stdlib/internal/basic/numeric-conversion.jai`) reads whitespace, the sign,
`inf`/`nan` and the hexadecimal prefix, then hands decimal text to `decimal_float_bits`
(`decimal-to-binary.jai`). That scanner keeps up to 19 significant digits in a `u64` `w` and a
decimal exponent `q`, so the value is `w * 10^q`, and tries in order:

1. **Exact float arithmetic.** When `w` and `10^|q|` are both exact in the target type
   (`w <= 2^53` and `|q| <= 22` for `float64`, `2^24` and `10` for `float32`), one IEEE multiply or
   divide rounds the exact product once. `"123.5"` takes this path.
2. **128-bit product (Eisel-Lemire).** `w` is shifted so its top bit is set and multiplied by a
   128-bit approximation of `5^q` from `FIVE_POWER_WORDS`. The top bits of the 192-bit product are
   the significand; the binary exponent comes from `floor(q * log2(10))`. The table entries round
   down, so the product is short of the exact value by less than two units of its second word. The
   path gives up when that shortfall could carry into the kept bits, or could hide that the exact
   value sits on a rounding boundary. Ties are only decided here when the entry is exact
   (`0 <= q <= 55`); for other `q` a value that is not rejected lies strictly inside an interval.
3. **Big integers.** More than 19 nonzero significant digits, exponents past `2^40`, hexadecimal
   floats and everything the earlier steps reject go to the original exact rational code in
   `numeric-conversion.jai`.

Every step either returns the correctly rounded bits or declines, so adding or removing a step never
changes a result, only its cost. Short decimals cost about 250 instructions per call against tens of
thousands on the big-integer path.

## How to change it

- `powers-of-five.jai` is generated: edit `tools/generators/generate_float_powers.py` and rerun it.
  `tools/generators/check_generated.py` fails when the checked-in table is stale. The fast path
  relies on every entry being a lower bound; a table rounded another way needs a new rejection test.
- To check a change, compare against the big-integer path and an independent oracle on many inputs:
  random bit patterns printed at every precision, exact midpoints between adjacent floats and their
  17-21 digit truncations, subnormal and overflow boundaries, and malformed text (the remainder
  returned must not change). Python's `float()` and `fractions.Fraction` make a good oracle for
  `float64` and, with a small rounding helper, for `float32`.
- `stdlib/Basic/tests/float-parsing.jai` and `float-roundtrip.jai` hold the edge cases.

## Configuration

None. The `$T` parameter (`float64` or `float32`) selects the constants at compile time.

## Dependencies

`multiply_64bits_with_overflow` (`integer128.jai`) for the 64x64 to 128-bit products. Nothing
outside Basic.
