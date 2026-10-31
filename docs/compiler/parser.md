# Parser

The recursive-descent parser in `crates/jaic-syntax/src/parser/` turns tokens from `lexer::lex` into an `ast::File`. It stops at the first error; there is no recovery. (The editor has a recovering variant, `parse_file_ide`.)

## How it works

- Keywords are plain identifiers, recognised only where expected (`kw()`, `at_kw("if")`). Directives arrive as `Tok::Directive(name)`.
- Types are ordinary expressions: `*T` is `Unary(Star)`, `[N] T` is `ArrayType`, a bodiless procedure header is `ProcType`.
- Expressions use precedence climbing (`expr.rs`) over `binary_op` ([operators](../language/operators.md)), then prefix operators and `cast`/`xx`, then a postfix loop (`.member`, `.*`, `[i]`, `(args)`, `.{..}`, `.[..]`).
- Parentheses aren't nodes. `(a + b)` parses to the inner expression with `Expr::parens` set. `Expr::span` leaves the parentheses out; `Expr::extent()` takes them in. Anything about how code was written (jailint, refactors) reads these, never source text. So `*Block.[a, b]` is an array of `*Block` while `*(Block.[a, b])` is an address-of.
- Files, struct bodies and blocks share one statement grammar (`stmt.rs`). Declarations (`decl.rs`) are found by lookahead: `name {, name} :`, `::` or `:=`.
- `null`, `true` and `false` always parse as literals, so variables with those names are rejected. Struct members may use them.
- A statement needs `;` unless the previous token closed a brace body, a here-string or `{} #flags`.
- Here-strings keep every line ending of the body: `#string END\nabc\nEND` is `"abc\n"`.

Lookahead decisions:

- `(` starts a procedure header if the matching `)` is followed by `->`, `=>` or a header directive, or the interior is empty or starts like a parameter (`paren_starts_header`).
- `.{` is one token, so `T.{..}` is always a literal. A bare `{1, .A}` in expression position is a literal (`brace_is_struct_literal`) unless it has a `;`, starts with a statement keyword or declaration, or follows `=>`.
- `if x == {` is a switch: `peek_binary_op` refuses `==` before `{`.
- `a=, b := f()` is one `Decl` with `existing` set per name.
- `cast(T)` / `xx` reach is `parse_cast_value` ([casts](../language/casts-and-conversions.md#how-far-a-prefix-cast-reaches)).

## How to change it

- Expression directive: an arm in `parse_directive_expr` (`directive.rs`) and an `ExprKind` variant. Statement directive: `parse_directive_stmt`.
- Procedure flag: `apply_directive_flag` plus a `ProcFlags` field, or `OTHER_FLAGS`. Declaration flag: `DECL_FLAGS` in `decl.rs`.
- Add tests in `parser/tests.rs`. A form that holds sub-expressions also goes in `ast/walk.rs` (`children`), the walker jailint and the language server share.
- A new declaration form needs a case in `decl_class` (`parser/ide_facts.rs`) for [editor facts](language-server.md).

Node sizes are a memory budget, because the language server keeps every tree alive: `Expr` 56 bytes, `Stmt` 88, `Decl` 176, `Arg` 88 (`node_sizes_stay_small`). Box rare or large (over about 32 bytes) fields; don't raise the bounds. Boxed statement payloads are read as `StmtKind::While(w)` then `w.cond`. `Decl` keeps rare fields in `DeclRare`; use its accessors and `rare_mut()` / `trim_rare()`.

Gotchas:

- `continues_after_block` stops `(`, `[`, `*`, `-`, `+`, `<<` and `.` at the start of a line after `}` from continuing the previous expression.
- `AstId::fresh()` numbers are unique across threads but follow thread timing.

Corpus runs: `JAIC_CORPUS_ROOT=/path JAIC_CORPUS_STAGE=lex|parse cargo test -p jaic --test corpus -- --ignored --nocapture`.
