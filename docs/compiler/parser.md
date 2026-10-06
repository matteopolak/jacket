# Parser

## What it is

The recursive-descent parser in `crates/jaic/src/parser/`. It turns the tokens from `lexer::lex` into an `ast::File` and stops at the first error; there is no recovery.

```rust
let file = jaic::parser::parse_file(file_id, text)?; // Result<ast::File, Diagnostic>
```

## How it works

- Keywords are plain identifiers, recognised only where one is expected (`kw()`, `at_kw("if")`). Directives arrive as `Tok::Directive(name)`.
- Types are ordinary expressions: `*T` is `Unary(Star)`, `[N] T` is `ArrayType`, a bodiless procedure header is `ProcType`, and `struct`/`enum` literals are expressions.
- Expressions use precedence climbing (`expr.rs`) over Jai's table in `binary_op` ([operator precedence](../language/operators.md)), then prefix operators and `cast`/`xx`, then a postfix loop (`.member`, `.*`, `[i]`, `(args)`, `.{..}`, `.[..]`, `.(T)`).
- Files, struct bodies and blocks share one statement grammar (`stmt.rs`). Declarations (`decl.rs`) are recognised by lookahead: `name {, name} :`, `::` or `:=`.
- A statement needs `;` unless the previous token closed a brace body, a here-string, or `{} #flags` (`Parser::ends_block`). `@notes` after a statement attach to it.
- Here-strings (`here_string` in `lexer.rs`) keep every line ending of the body, including the one before the terminator line: `#string END\nabc\nEND` is `"abc\n"`. See [strings and literals](../language/strings-and-literals.md).

### Lookahead decisions

| Question | Rule |
| --- | --- |
| Is `(` a procedure header or a parenthesized expression? | `paren_starts_header`: a header if the matching `)` is followed by `->`, `=>` or a header directive (`#c_call`, ...), or the interior is empty or starts like a parameter (`name:`, `$T`, `using`, `..`, a top-level comma). |
| Struct literal or block? | `.{` is one token, so `T.{..}` is always a literal and `cond {` always a block. A bare `{1, .A}` or `{ a = 1 }` in expression position is a literal (`brace_is_struct_literal`) unless it holds a `;` at its own level, starts with a statement keyword (`if`, `for`, `return`, ...), a directive other than `#char`, a backtick or a declaration (`name :=`), or follows `=>` (a lambda body). |
| `if x == {` | `peek_binary_op` refuses `==` when `{` or `#complete {` follows, so the `if` parser sees a switch. |
| `[2]int.[1, 2]` | The element type is parsed without `.{`/`.[` postfix, so the literal applies to the whole array type. |
| `#run,stallable` | `parse_directive_flags`: a comma right after the directive, then an identifier. A few known flags (`distinct`, `file`, ...) may be spaced. |
| `#library, system, link_always "Metal"` | Flags before the name string may be spaced after the first (`parse_unknown_directive`). |
| `#if #complete X == {` | `parse_static_if` skips `#complete`; a static switch has no completeness check. |
| `(-cast,no_check(int) x)` | `has_top_level_comma` ignores a comma between a cast keyword and its modifier, so this stays an expression. |
| `arrow.to\,` | `\` after an identifier joins lines (`ident_with_separators` in the lexer); a trailing one before a non-identifier is dropped. |
| Commas after a return type | Inside argument and parameter lists (`in_list`) a comma ends the return type instead of adding a return value. |
| `a=, b := f()`, `a:, b = f()` | One `Decl` with `existing` set per name. A `name:` marker only matters in an assignment (`=`); in `ok, shader:, time := f()` every name is declared. An existing entry may be a place (`ok:, t.str = f()`, `t.a[i]=, n := g()`): `decl_ahead` skips members, indexes and `.*` (`place_end`), the place goes in `Decl::targets` and its root variable in `names`. Without a marker, `a.b, c = f()` stays a plain assignment. |
| How far `cast(T)` and `xx` reach | `parse_cast_value`: a unary operand plus any chain of bitwise and shift operators (the levels above `CAST_PREC`). See [casts](../language/casts-and-conversions.md#how-far-a-prefix-cast-reaches). `cast(T, x)` and `.(T)` are unaffected. |

## How to change it

- Expression-level directive: an arm in `parse_directive_expr` (`directive.rs`) and an `ExprKind` variant in `ast.rs`.
- Statement-level directive: an arm in `parse_directive_stmt` (`directive_stmt.rs`).
- Procedure flag: `apply_directive_flag` in `procedure.rs`, plus a `ProcFlags` field, or add the name to `OTHER_FLAGS` to keep it by name only.
- Declaration flag: `DECL_FLAGS` in `decl.rs`. Flags on the next line are not consumed; they start the next statement.

Gotchas:

- `continues_after_block` stops `(`, `[`, `*`, `-`, `+`, `<<` and `.` at the start of a line right after `}`, so `Foo :: struct {}` followed by `*p = 1;` stays two statements.
- `AstId::fresh()` counters are per thread.

## Configuration

`crates/jaic/tests/corpus.rs` lexes or parses whole corpora. `JAIC_CORPUS_ROOT` is the checkout holding them (default: this repo) and `JAIC_CORPUS_STAGE` is `lex` or `parse`:

```sh
JAIC_CORPUS_ROOT=/path/to/jai JAIC_CORPUS_STAGE=parse \
  cargo test -p jaic --test corpus -- --ignored --nocapture
```

## Dependencies

`jaic::lexer`, `jaic::ast`, `jaic::intern`, `jaic::source`.
