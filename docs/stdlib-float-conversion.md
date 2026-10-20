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

## Formatting

`emit_float` (`format-float.jai`) turns a `FormatFloat` into text. The digits come from
`portable_float` semantics: printf-style `f`, `e` and `g` conversions of the exact binary value,
rounded half to even on its exact decimal expansion (`round_up` rounds toward infinity instead,
for shortest mode below). `fast_float_text` (`binary-to-decimal.jai`) produces the same text
without big integers when the numbers fit:

- The value is `m * 2^b` with `m < 2^53`. The digits wanted are `round(m * 2^b * 10^s)`: `s` is the
  precision for `f`, and `significant - 1 - k` for `e`/`g`, where `k` is the decimal exponent.
- `scaled_round` splits `10^s` into `5^s * 2^s`, multiplies or divides `m` by `5^|s|` (up to `5^27`,
  one word) and shifts by the combined power of two. The quotient and the exact remainder, compared
  with one half, give a correctly rounded integer. Anything that would not fit in 128 bits, or a
  result of 2^64 or more, returns `ok = false`.
- `leading_digits` estimates `k` as `floor(top * log10(2))` from the binary exponent and corrects it
  by one when the scaled result shows the estimate was low. A result of exactly `10^significant` is
  a carry out of the last digit and moves the exponent.
- The digits are laid out into a 48-byte stack buffer exactly as `decimal_layout`,
  `scientific_layout` and `trim_fraction` would, so `emit_float` allocates nothing on this path.

When `fast_float_text` declines (precision past 27 fraction digits or 19 significant digits, very
large or very small magnitudes), `portable_float` builds the full exact decimal expansion.

Shortest mode (`shortest_text`) bisects on the digit count for the smallest `g` precision whose text
parses back to the same value; normal powers of two also try the rounded-up text, because their
interval below is half the size of the one above. Candidates are formatted into stack buffers and
checked with the fast parser; each costs roughly 1,300 instructions.

Rough costs per call at `-O2` (including `print_to_builder`): `formatFloat(x, trailing_width = 3)`
about 1,050 instructions, default `print("%", x)` about 1,200, shortest mode about 6,600.

## How to change formatting

- `fast_float_text` must return exactly what `portable_float` returns for the same arguments, or
  decline. Compare both over many bit patterns (uniform random bits, typical magnitudes, exact
  ties such as `k / 2^j`, subnormals, values near `2^53` and `2^64`) and every `trailing_width`
  from -1 to 30 in each mode, for `float64` and `float32`.
- Raising the limits needs wider arithmetic: `FIVE_POWERS` stops at `5^27` because `m * 5^27` must
  stay under 2^117, and the `10^significant` limit table stops at `10^19`.
- `stdlib/Basic/tests/format-floats.jai` holds the edge cases.

## How to change parsing

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

`multiply_64bits_with_overflow` (`integer128.jai`) for the 64x64 to 128-bit products in both
directions. Nothing outside Basic.
