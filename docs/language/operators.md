# Operator precedence

How an expression with several operators groups. Jai's table is not C's: the bitwise and shift operators share one level that binds tighter than `*`, and `%` sits between `*`/`/` and `+`/`-`.

Tightest first. Every binary level is left associative (`8 / 2 / 2` is 2, `1 << 1 << 2` is 8) {#ops.1}.

| Level | Operators | Example |
| --- | --- | --- |
| postfix | `.x` `[i]` `f()` `.*` `.{}` `.[]` `.(T)` | `-p.x` is `-(p.x)`, `*a.b` is `*(a.b)` {#ops.2} |
| prefix | `-` `+` `!` `~` `*` (address) `<<` (dereference) | `~a & b` is `(~a) & b` {#ops.3} |
| bitwise | `&` `\|` `^` `<<` `>>` `<<<` `>>>` | `1 \| 2 & 4` is `(1 \| 2) & 4` = 0 {#ops.4} |
| prefix cast | `cast(T)` `xx` | `cast(u32) b << 4 \| 1` is `cast(u32) ((b << 4) \| 1)` {#ops.5} |
| multiplicative | `*` `/` | `1 << 2 * 3` is `(1 << 2) * 3` = 12 {#ops.6} |
| remainder | `%` | `10 % 3 * 2` is `10 % (3 * 2)` = 4 {#ops.7} |
| additive | `+` `-` | `tab - col % tab` is `tab - (col % tab)` {#ops.8} |
| relational | `<` `<=` `>` `>=` | `1 + 2 < 4` is `(1 + 2) < 4` {#ops.9} |
| equality | `==` `!=` | `0x0F & 0x33 == 0x03` is true {#ops.10} |
| and | `&&` | `a == b && c` is `(a == b) && c` {#ops.11} |
| or | `\|\|` | `a \|\| b && c` is `a \|\| (b && c)` {#ops.12} |

## Notes

- A prefix cast's operand runs through the bitwise level: `cast(float) (hex >> 16) & 0xFF` casts the masked integer {#ops.13}, while `cast(s64) p - cast(s64) q` subtracts two integers {#ops.14}. See [casts](casts-and-conversions.md).
- The cast doesn't type its operand's literals: `cast(float32) (0 - w)` with `w: u16` subtracts in `u16`, because an untyped operand takes the other operand's type, never the context's ([numbers](numbers.md)) {#ops.17}.
- Parenthesize shifts combined with `|`: `b | a << 8` means `(b | a) << 8` {#ops.15}.
- A procedure name compared with `null` (`proc != null`) is the procedure's own address, so it is never null {#ops.18}.
- Prefix `-` and `~` need an integer, enum or float operand, or an `operator -`/`operator ~` overload. On `Type`, `bool`, pointers or strings they are errors (`cannot negate Type`); use `!` for `bool` {#ops.16}.

## How to change it

The table is `binary_op` in `parser/expr.rs` (`BITWISE_PREC`, `CAST_PREC`). Changing a level silently changes existing code: parse the stdlib, tests and examples with both tables, list every expression whose tree differs, and parenthesize those. Update the parser tests (`binary_precedence*`) and `tests/compiler/regressions/operator-precedence.jai`.

`stdlib/internal/program-print` and `Bindings_Generator` keep their own copy of the table. jaifmt needs no change.
