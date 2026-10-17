# Operator precedence

## What it is

How jaic groups an expression with several operators. Jai's table is not C's: the bitwise and shift operators share one level that binds tighter than `*`, and `%` sits between `*`/`/` and `+`/`-`. The table is inferred from observed Jai behaviour, since no precedence specification is published.

## How it works

Tightest first; every binary level is left associative (`8 / 2 / 2` is 2).

| Level | Operators | Example |
| --- | --- | --- |
| postfix | `.x` `[i]` `f()` `.*` `.{}` `.[]` `.(T)` | `-p.x` is `-(p.x)` |
| prefix | `-` `+` `!` `~` `*` `<<` | `~a & b` is `(~a) & b` |
| bitwise | `&` `\|` `^` `<<` `>>` `<<<` `>>>` | `1 \| 2 & 4` is `(1 \| 2) & 4` |
| prefix cast | `cast(T)` `xx` | `cast(u32) b << 4 \| 1` casts the whole `(b << 4) \| 1` |
| multiplicative | `*` `/` | `1 << 2 * 3` is `(1 << 2) * 3` = 12 |
| remainder | `%` | `10 % 3 * 2` is `10 % (3 * 2)` = 4 |
| additive | `+` `-` | `tab - col % tab` |
| relational | `<` `<=` `>` `>=` | |
| equality | `==` `!=` | `0x0F & 0x33 == 0x03` is true |
| and, or | `&&`, `\|\|` | `a \|\| b && c` is `a \|\| (b && c)` |

- A cast does not type its operand's literals: `cast(float32) (0 - w)` with `w: u16` subtracts in `u16` ([numbers](numbers.md), [casts](casts-and-conversions.md)).
- Parenthesize shifts combined with `|`: `b | a << 8` is `(b | a) << 8`.
- Prefix `-` and `~` need an integer, enum or float (or an `operator -`/`operator ~` overload); on `Type`, `bool`, pointers or strings they are errors. Use `!` for `bool`.
- Not settled by observation, kept as is: `==`/`!=` below relational, `%` against `/`, rotations grouped with shifts, `!` grouped with prefix.

## How to change it

The table is `binary_op` in `parser/expr.rs` (`BITWISE_PREC`, `CAST_PREC`; also `parse_binary`, `parse_cast_value`, `parse_unary`). Changing a level silently changes existing code, so parse the stdlib, tests and examples with both tables, list every expression whose tree differs, and parenthesize those sites. Update the parser tests (`binary_precedence*`, `prefix_operators_bind_tighter_than_binary_ones`) and `tests/stdlib/operator-precedence.jai`, `cast-operand-precedence.jai`.

Two stdlib modules keep their own copy of the table: `stdlib/internal/program-print` (where printed code needs parentheses) and `Bindings_Generator` (which parenthesizes C macro bodies). jaifmt needs no change.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Tightest first. Every binary level is left associative (`8 / 2 / 2` is 2, `1 << 1 << 2` is 8) {#ops.1}.
- | postfix | `.x` `[i]` `f()` `.*` `.{}` `.[]` `.(T)` | `-p.x` is `-(p.x)`, `*a.b` is `*(a.b)` {#ops.2} |
- | prefix | `-` `+` `!` `~` `*` (address) `<<` (dereference) | `~a & b` is `(~a) & b` {#ops.3} |
- | bitwise | `&` `\|` `^` `<<` `>>` `<<<` `>>>` | `1 \| 2 & 4` is `(1 \| 2) & 4` = 0 {#ops.4} |
- | prefix cast | `cast(T)` `xx` | `cast(u32) b << 4 \| 1` is `cast(u32) ((b << 4) \| 1)` {#ops.5} |
- | multiplicative | `*` `/` | `1 << 2 * 3` is `(1 << 2) * 3` = 12 {#ops.6} |
- | remainder | `%` | `10 % 3 * 2` is `10 % (3 * 2)` = 4 {#ops.7} |
- | additive | `+` `-` | `tab - col % tab` is `tab - (col % tab)` {#ops.8} |
- | relational | `<` `<=` `>` `>=` | `1 + 2 < 4` is `(1 + 2) < 4` {#ops.9} |
- | equality | `==` `!=` | `0x0F & 0x33 == 0x03` is true {#ops.10} |
- | and | `&&` | `a == b && c` is `(a == b) && c` {#ops.11} |
- | or | `\|\|` | `a \|\| b && c` is `a \|\| (b && c)` {#ops.12} |
- A prefix cast's operand runs through the bitwise level: `cast(float) (hex >> 16) & 0xFF` casts the masked integer {#ops.13}, while `cast(s64) p - cast(s64) q` subtracts two integers {#ops.14}; see [casts and conversions](casts-and-conversions.md). The cast does not type its operand's literals: `cast(float32) (0 - w)` with `w: u16` subtracts in `u16`, because an untyped operand takes the other operand's type, never the type of the context around the operator ([numbers](numbers.md)) {#ops.17}.
- Write shifts combined with `|` with parentheses: `(a << 8) | b`. `a << 8 | b` happens to work because the shift comes first, but `b | a << 8` means `(b | a) << 8` {#ops.15}.
- Prefix `-` and `~` need an integer, enum or float operand, or an `operator -`/`operator ~` overload for a struct. On a `Type`, `bool`, pointer or string they are errors (`cannot negate Type`, `cannot bit-complement bool`); use `!` to invert a `bool` {#ops.16}.
