# Float conversion in Basic

`string_to_float64`, `string_to_float` and the float cases of `print`/`formatFloat` convert between binary floats and decimal text. Every result is exact: parsing returns the correctly rounded float (ties to even), and formatting prints the correctly rounded digits of the exact binary value. Each direction has fast paths for common inputs and one big-integer path that handles everything.

## Parsing

`parse_float_number` (`stdlib/internal/basic/numeric-conversion.jai`) reads whitespace, sign, `inf`/`nan` and the hex prefix, then hands decimal text to `decimal_float_bits` (`decimal-to-binary.jai`). It keeps up to 19 significant digits in a `u64` `w` and a decimal exponent `q`, so the value is `w * 10^q`, and tries in order:

1. **Exact float arithmetic.** If `w` and `10^|q|` are both exact in the target type (`w <= 2^53`, `|q| <= 22` for `float64`), one IEEE multiply or divide rounds once. `"123.5"` takes this path.
2. **Eisel-Lemire.** Multiply the normalised `w` by a 128-bit approximation of `5^q` (`FIVE_POWER_WORDS`) and take the top bits. The table rounds down, so the path declines when the shortfall could reach the kept bits or hide a rounding boundary.
3. **Big integers.** More than 19 significant digits, huge exponents, hex floats and anything declined go to the exact rational code in `numeric-conversion.jai`.

Each step returns the correctly rounded bits or declines, so adding or removing a step changes cost, never results. Short decimals cost about 240 instructions against tens of thousands on the big-integer path.

## Formatting

`emit_float` (`format-float.jai`) turns a `FormatFloat` into text. The reference semantics are `portable_float`: printf-style `f`, `e`, `g` of the exact binary value, rounded half to even. `fast_float_text` (`binary-to-decimal.jai`) produces the same text without big integers when the numbers fit:

- The value is `m * 2^b` with `m < 2^53`. The wanted digits are `round(m * 2^b * 10^s)`. `scaled_round` multiplies or divides `m` by `5^|s|` (up to `5^27`) and shifts, then compares the exact remainder with one half. Anything past 128 bits returns `ok = false`.
- `leading_digits` estimates the decimal exponent from the binary one and corrects it by one when the scaled result shows it was low.
- Digits are laid out in a 48-byte stack buffer, so this path allocates nothing.

When it declines (precision past 27 fraction digits or 19 significant digits, extreme magnitudes), `portable_float` builds the full decimal expansion.

**Shortest mode** (`shortest_text`) bisects on digit count for the smallest `g` precision that reads back as the same value (powers of two also try the rounded-up text). `digits_read_back` checks each try with 128-bit integers against the rounding interval, without writing or parsing text; `text_reads_back` does the same on `Natural` big integers when numbers do not fit. It never calls the parser, so a program that prints does not compile the parsing code.

Rough costs at `-O2`: `formatFloat(x, trailing_width = 3)` about 800 instructions, `print("%", x)` about 900, shortest mode about 3,100.

## How to change it

- `fast_float_text` must return exactly what `portable_float` returns, or decline. Compare over many bit patterns (random bits, typical magnitudes, exact ties like `k / 2^j`, subnormals, values near `2^53` and `2^64`) and every `trailing_width` from -1 to 30 in each mode, for `float64` and `float32`.
- Raising the limits needs wider arithmetic: `FIVE_POWERS` stops at `5^27` because `m * 5^27` must stay under 2^117, and the limit table at `10^19`.
- Do not call the parser from formatting code: everything `print` reaches is compiled into every program that prints. Check shortest output against format-then-parse.
- `powers-of-five.jai` is generated: edit `tools/generators/generate_float_powers.py` and rerun it. The fast path relies on every entry being a lower bound; a table rounded another way needs a new rejection test.
- To check a parsing change, compare against the big-integer path and an independent oracle (Python's `float()` and `fractions.Fraction`): random bit patterns at every precision, midpoints between adjacent floats and their 17-21 digit truncations, subnormal and overflow edges, and malformed text.
- Edge cases live in `stdlib/Basic/tests/format-floats.jai`, `float-parsing.jai` and `float-roundtrip.jai`.

The `$T` parameter (`float64` or `float32`) selects constants at compile time. Dependencies: `multiply_64bits_with_overflow` (`integer128.jai`); nothing outside Basic.
