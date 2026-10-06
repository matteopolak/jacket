# Operator precedence

## What it is

How jaic groups an expression with several operators. Jai's table is not C's: the bitwise and shift operators share one level that binds tighter than `*`, and `%` sits between `*`/`/` and `+`/`-`. There is no published specification, so the table was reconstructed from evidence, listed below.

## How it works

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

A prefix cast's operand runs through the bitwise level: `cast(float) (hex >> 16) & 0xFF` casts the masked integer {#ops.13}, while `cast(s64) p - cast(s64) q` subtracts two integers {#ops.14}; see [casts and conversions](casts-and-conversions.md).

Write shifts combined with `|` with parentheses: `(a << 8) | b`. `a << 8 | b` happens to work because the shift comes first, but `b | a << 8` means `(b | a) << 8` {#ops.15}.

### Evidence

The strongest source is the recorded runtime output at the bottom of open-jai's `utils/stress.jai`, produced by Jai (its `expect` comments are wrong where they disagree with the output). The rest comes from code in the pinned corpus and the public source files that only type-checks or makes sense one way, and from the official changelog.

| Relation | Evidence |
| --- | --- |
| shifts above `+` and `*` | recorded `1 << 2 + 3 == 7`, `8 >> 1 + 1 == 5`, `1 << 2 * 3 == 12`, `1 + 2 * 3 << 1 == 13` |
| `&` `\|` `^` above `*` and `+` | a prefix cast absorbs `&` and `\|` (ui_builder, jai-utils, allocator pointer masks) but stops at `*` and `+` (the the language version changelog) |
| `&` and `\|` on one level | recorded `1 \| 2 & 4 == 0`; chess-jai `get_queen(..) \| get_rook(..) & ep_rank` filters both to the rank |
| shifts on the same level as `\|` | `Window_Creation` notes that `(b << 16) \| (g << 8) \| r` needs its parentheses; `Int128` writes `a.high << x \| ...` |
| `%` below `*`, above `+` `-` | recorded `10 % 3 * 2 == 4`; tab stops (`tab_size - col % tab_size`) in focus and Simp; recorded output of stress.jai's `reverse_int` and Caesar ciphers |
| `*` and `/` on one level | `(n + 7) / 8 * 8`, `PI / 180.0 * 72.0` |
| bitwise above comparisons | recorded `0x0F & 0x33 == 0x03` is true |
| `&&` above `\|\|` | many character-class tests across the corpus |
| prefix above bitwise | 0.0.081 changelog (`~a & b` is `(~a) & b`); `(word - ONES) & ~word & HIGH` |
| postfix above prefix | changelog (`-Thing.{5}` is `-(Thing.{5})`); `*a.b` everywhere |
| left associativity | recorded `8 / 2 / 2`, `100 - 50 - 25`, `1 << 1 << 2` |

Not settled by evidence, kept as is: `==`/`!=` below the relational operators (no code mixes them unparenthesized), `%` against `/`, the rotations (grouped with shifts), and `!` against bitwise (grouped with prefix).

The only real code that reads better with C's grouping is Vk-Engine's `gizmo.jai` `1 << axis0 | 1 << axis1`, which under this table computes `((1 << axis0) | 1) << axis1`. It sits on a keyboard-only path and is most likely an upstream bug.

## How to change it

The table is `binary_op` in `parser/expr.rs`; `BITWISE_PREC` is the shared bitwise level and `CAST_PREC` the prefix cast level. Changing a level silently changes the meaning of existing code, so:

1. Parse everything twice, with the old and new tables, and list every expression whose tree differs. (A throwaway tool did this: a thread-local table override in `expr.rs` plus a binary comparing `(span, op)` lists. It is not in the repo.)
2. Run it over `stdlib/`, `tests/`, `examples/`, `tools/`, `benchmarks/`, and parenthesize each site to keep its meaning.
3. Update the parser tests (`binary_precedence*`, `prefix_operators_bind_tighter_than_binary_ones`) and `tests/stdlib/operator-precedence.jai`.

Two stdlib modules keep their own copy of the table and must change with it:

- `Program_Print` (`pp_precedence`, `PP_CAST_LEVEL`) decides where printed code needs parentheses.
- `Bindings_Generator` (`jai_binding` in `convert.jai`) parenthesizes C macro bodies where Jai groups differently from C.

jaifmt never adds or removes tokens and spaces all binary operators alike, so it needs no change.

## Dependencies

`parser/expr.rs` (`binary_op`, `parse_binary`, `parse_cast_value`, `parse_unary`), `stdlib/Program_Print/module.jai`, `stdlib/Bindings_Generator/convert.jai`. Tests: `crates/jaic/src/parser/tests.rs`, `tests/stdlib/operator-precedence.jai`, `cast-operand-precedence.jai`, `program-print-expressions.jai`, `bindings-generator-c.jai`.
