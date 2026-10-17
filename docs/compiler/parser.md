# Parser

## What it is

The recursive-descent parser in `crates/jaic/src/parser/`. It turns tokens from `lexer::lex` into an `ast::File` and stops at the first error; there is no recovery.

## How it works

- Keywords are plain identifiers, recognised only where expected (`kw()`, `at_kw("if")`). Directives arrive as `Tok::Directive(name)`.
- Types are ordinary expressions: `*T` is `Unary(Star)`, `[N] T` is `ArrayType`, a bodiless procedure header is `ProcType`.
- Expressions use precedence climbing (`expr.rs`) over `binary_op` ([operator precedence](../language/operators.md)), then prefix operators and `cast`/`xx`, then a postfix loop (`.member`, `.*`, `[i]`, `(args)`, `.{..}`, `.[..]`).
- Files, struct bodies and blocks share one statement grammar (`stmt.rs`). Declarations (`decl.rs`) are recognised by lookahead: `name {, name} :`, `::` or `:=`.
- A statement needs `;` unless the previous token closed a brace body, a here-string or `{} #flags` (`Parser::ends_block`).
- A BOM and then a `#!` line at the start of a file are skipped by `Lexer::run`; a `#!` elsewhere is an error.
- Here-strings keep every line ending of the body, including the one before the terminator: `#string END\nabc\nEND` is `"abc\n"`.

### Lookahead decisions

| Question | Rule |
| --- | --- |
| `(` header or parenthesized expression? | `paren_starts_header`: a header if the matching `)` is followed by `->`, `=>` or a header directive, or the interior is empty or starts like a parameter (`name:`, `$T`, `using`, `..`, a top-level comma). |
| Struct literal or block? | `.{` is one token, so `T.{..}` is always a literal and `cond {` a block. A bare `{1, .A}` in expression position is a literal (`brace_is_struct_literal`) unless it has a `;` at its level, starts with a statement keyword, a non-`#char` directive, a backtick or a declaration, or follows `=>`. |
| `if x == {` | `peek_binary_op` refuses `==` when `{` or `#complete {` follows, so it parses as a switch. |
| `a=, b := f()`, `ok:, t.str = f()` | One `Decl` with `existing` set per name; a place goes in `Decl::targets` and its root in `names` (`decl_ahead`, `place_end`). A `name:` marker only matters in an assignment. |
| Reach of `cast(T)` / `xx` | `parse_cast_value`: a unary operand plus bitwise and shift operators ([casts](../language/casts-and-conversions.md#how-far-a-prefix-cast-reaches)). |

## How to change it

- **Expression directive**: an arm in `parse_directive_expr` (`directive.rs`) and an `ExprKind` variant in `ast.rs`.
- **Statement directive**: an arm in `parse_directive_stmt` (`directive_stmt.rs`).
- **Procedure flag**: `apply_directive_flag` in `procedure.rs` plus a `ProcFlags` field, or add the name to `OTHER_FLAGS`. **Declaration flag**: `DECL_FLAGS` in `decl.rs`.
- Add syntax to `lexer.rs`, `parser/` and `ast.rs` with tests in `parser/tests.rs`.

### Node sizes

The language server keeps the syntax tree of every compile alive, so node size is a memory budget: `Expr` 56 bytes, `Stmt` 88, `Decl` 176, `Arg` 88, held by `node_sizes_stay_small` in `ast.rs`. When you add a field or variant, box it if it is rare or over about 32 bytes; do not raise the bounds.

- Rare large payloads are boxed (`StmtKind::While(w)`, `Switch`, `StaticIf`, ...). Match `StmtKind::While(w)` and read `w.cond`; a boxed payload cannot be destructured in the pattern.
- `Decl` keeps common fields and moves the rest into `rare: Option<Box<DeclRare>>`. Read through accessors (`decl.extra_values()`, `decl.align()`); fill with `rare_mut()` and finish with `trim_rare()`.
- `Notes` is one nullable pointer that reads as a `[Note]` slice. `ExprKind::Int` holds an `Int128` (two `u64` words) so `ExprKind` needs only 8-byte alignment.

Gotchas: `continues_after_block` stops `(`, `[`, `*`, `-`, `+`, `<<` and `.` at the start of a line right after `}`, so `Foo :: struct {}` then `*p = 1;` stays two statements. `AstId::fresh()` counters are per thread.

## Configuration

`crates/jaic/tests/corpus.rs` lexes or parses whole corpora: `JAIC_CORPUS_ROOT=/path JAIC_CORPUS_STAGE=lex|parse cargo test -p jaic --test corpus -- --ignored --nocapture` (root defaults to this repo).

## Dependencies

`jaic::lexer`, `jaic::ast`, `jaic::intern`, `jaic::source`.
