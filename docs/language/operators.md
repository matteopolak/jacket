# Operator precedence

## What it is

How jaic groups an expression with several operators. Jai's table is not C's: the bitwise and shift operators bind tighter than `*`, they share one level, and `%` sits between `*`/`/` and `+`/`-`. The table below was reconstructed from evidence (recorded output of the real compiler, code that only works one way, the reference CHANGELOG), because jaic has no access to the real compiler.

## How it works

Tightest first. Every binary level is left associative (`8 / 2 / 2` is 2, `1 << 1 << 2` is 8).

| Level | Operators | Example |
| --- | --- | --- |
| postfix | `.x` `[i]` `f()` `.*` `.{}` `.[]` `.(T)` | `-p.x` is `-(p.x)`, `*a.b` is `*(a.b)` |
| prefix | `-` `+` `!` `~` `*` (address) `<<` (dereference) | `~a & b` is `(~a) & b` |
| bitwise | `&` `\|` `^` `<<` `>>` `<<<` `>>>` | `1 \| 2 & 4` is `(1 \| 2) & 4` = 0 |
| prefix cast | `cast(T)` `xx` | `cast(u32) b << 4 \| 1` is `cast(u32) ((b << 4) \| 1)` |
| multiplicative | `*` `/` | `1 << 2 * 3` is `(1 << 2) * 3` = 12 |
| remainder | `%` | `10 % 3 * 2` is `10 % (3 * 2)` = 4 |
| additive | `+` `-` | `tab - col % tab` is `tab - (col % tab)` |
| relational | `<` `<=` `>` `>=` | |
| equality | `==` `!=` | `0x0F & 0x33 == 0x03` is true |
| and | `&&` | |
| or | `\|\|` | `a \|\| b && c` is `a \|\| (b && c)` |

A prefix cast is a unary operator whose operand runs through the bitwise level: `cast(float) (hex >> 16) & 0xFF` casts the masked integer, while `cast(s64) p - cast(s64) q` subtracts two integers. Details in [casts-and-conversions.md](casts-and-conversions.md).

Write parentheses when combining a shift with `|` the C way: `(a << 8) | b` is needed, `a << 8 | b` happens to work only because the shift comes first, and `b | a << 8` means `(b | a) << 8`.

### Evidence

Sources: the recorded runtime output at the bottom of open-jai's `utils/stress.jai` (produced by the real compiler: its `expect` comments are wrong where the output disagrees with C), the pinned corpus (`corpus/upstream`), and the public source files's modules, examples and CHANGELOG. Sites were found by parsing the whole corpus under two candidate tables and listing every expression that groups differently (the counts are distinct source lines; copies of the same file are counted once).

| Relation | Evidence | Against |
| --- | --- | --- |
| shifts above `+` | recorded `1 << 2 + 3 == 7`, `8 >> 1 + 1 == 5` (2) | none |
| shifts above `*` | recorded `1 << 2 * 3 == 12`, `1 + 2 * 3 << 1 == 13` (2) | none |
| `&` `\|` `^` above `*` and `+` | a prefix cast takes `&` into its value (ui_builder 1, jai-utils 4, about 20 reference allocator lines) and `xx` takes `&` and `\|` (2), but a cast stops at `*` (reference invaders 1) and `+` (CHANGELOG the language version), so `&` binds tighter than both; `\|` and `^` share `&`'s level (next rows); AST_Utils `(n * s * 53 / count) & s + a` (3, intent) | none |
| `&` not above `\|` | recorded `1 \| 2 & 4 == 0` (1); chess-jai `get_queen(..) \| get_rook(..) & ep_rank` (1, filters both to the rank) | none |
| shifts not above `\|` | reference `Window_Creation/windows.jai` notes that `(b << 16) \| (g << 8) \| r` needs its parentheses (1) | Vk-Engine `gizmo.jai` `1 << axis0 \| 1 << axis1` (2); reference `Socket/generated_*.jai` `TCPOPT_NOP<<24\|...` (4, emitted mechanically from C headers) |
| shifts not below `\|` | reference `Basic/Int128.jai` `a.high << x \| ...` (3), `string_to_float.jai` (1), `d3d12` (1); uniform `id << 1 \| 1` (1) | none |
| `%` below `*` | recorded `10 % 3 * 2 == 4` (1) | none |
| `%` above `+` `-` | 29 lines: focus and Simp tab stops (`tab_size - col % tab_size`, 8), Photon (3), chess-jai (4), reference `md5.jai` padding, `Bindings_Generator`, allocator tests (5), KodaJai, VR example; stress.jai `reverse_int` and two Caesar ciphers with recorded output (5) | none |
| `*` and `/` one level | 23 lines: `(n + 7) / 8 * 8`, `PI / 180.0 * 72.0`, raymath remap, KodaJai geometry | none |
| bitwise above comparisons | recorded `0x0F & 0x33 == 0x03` is true (1) | none |
| `&&` above `\|\|` | about 20 distinct expressions (focus character classes, Jails, jai_parser, uniform, reference `Apollo_Time`, `Debug`, `GetRect`, `POSIX_old`, `rpmalloc`) | none |
| prefix above bitwise | CHANGELOG 0.0.081 (`~a & b` is `(~a) & b`, same level as unary `-`); jai_parser `(word - ONES) & ~word & HIGH` (1); chess-jai `& ~occupied` (2); reference Int128 `-shift & 63`, `-shift >> 63` (2) | none |
| postfix above prefix | CHANGELOG (`-Thing.{5}` is `-(Thing.{5})`); `*a.b` throughout | none |
| left associativity | recorded `8 / 2 / 2`, `100 - 50 - 25`, `100 / 5 / 2`, `1 << 1 << 2` (4) | none |

Not decided by evidence, kept as before: `==`/`!=` one level below `<`/`<=`/`>`/`>=` (no corpus expression mixes them unparenthesized), `%` against `/` (follows from `%` below `*` with `*` and `/` on one level, but no direct example), the rotations `<<<`/`>>>` (grouped with the shifts), and `!` against the bitwise operators (grouped with the other prefix operators).

The Vk-Engine gizmo lines are the only real code that reads better with C's grouping. Under the table above they compute `((1 << axis0) | 1) << axis1`; that would be an upstream bug in a keyboard-only gizmo path, which is weaker evidence than the reference comment and the recorded `1 | 2 & 4`.

## How to change it

The table is `binary_op` in `crates/jaic/src/parser/expr.rs`; `CAST_PREC` is the level of a prefix cast (`parse_cast_value` parses the cast's operand with it) and `BITWISE_PREC` the shared bitwise level. Changing a level changes the meaning of existing code silently, so:

1. Parse the whole tree twice, old and new table, and list every expression whose binary nodes differ. A throwaway version of that tool (a thread-local table override in `expr.rs` plus a small binary that walks directories and compares the recorded `(span, op)` lists) found the sites listed here; it is not kept in the repository.
2. Run it over `stdlib/`, `tests/`, `examples/`, `tools/`, `benchmarks/`, and parenthesize each site to keep its intended meaning.
3. Update the parser tests (`binary_precedence*`, `prefix_operators_bind_tighter_than_binary_ones`) and `tests/stdlib/operator-precedence.jai`.

Our own code was rewritten once already: 26 lines in 13 files (UTF-8 encoders in `Jai_Lexer/scanner.jai` and `Keymap`, byte assembly in `Socket` and `executable_formats`, `%` before `/` or `*` in `Apollo_Time`, `platform-time.jai` and `Process/windows.jai`, the generated `Socket` constants, a `jaifmt` spacing case and `tests/stdlib/asm-simd-extended.jai`).

Two stdlib modules hold their own copy of the table and must change with it: `Program_Print` (`pp_precedence`, `PP_CAST_LEVEL`) decides where printed code needs parentheses, and `Bindings_Generator` (`jai_binding` in `convert.jai`) parenthesizes C macro bodies wherever Jai would group them differently from C.

jaifmt does not reason about precedence: it never adds or removes tokens and spaces every binary operator the same way, so it needs no change when the table does.

## Configuration

None.

## Dependencies

`crates/jaic/src/parser/expr.rs` (`binary_op`, `parse_binary`, `parse_cast_value`, `parse_unary`); the copies in `stdlib/Program_Print/module.jai` and `stdlib/Bindings_Generator/convert.jai`. Tests: `crates/jaic/src/parser/tests.rs`, `tests/stdlib/operator-precedence.jai`, `tests/stdlib/cast-operand-precedence.jai`, `tests/stdlib/program-print-expressions.jai`, `tests/stdlib/bindings-generator-c.jai`.
