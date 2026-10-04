# jaic parser

## What it is

The recursive-descent parser of the new compiler core (`crates/jaic/src/parser/`). It turns the token
vector from `lexer::lex` into the `ast::File` contract and stops at the first error (no recovery).

```rust
let file = jaic::parser::parse_file(file_id, text)?; // Result<ast::File, Diagnostic>
```

## How it works

- Keywords are plain identifiers, so each keyword is recognised only where one is expected (`kw()`,
  `at_kw("if")`). Directives arrive as `Tok::Directive(name)`.
- Types are ordinary expressions: `*T` is `Unary(Star)`, `[N] T` is `ArrayType`, procedure headers
  without a body are `ProcType`, `struct`/`enum` literals are expressions.
- Expressions use precedence climbing (`expr.rs`); prefix operators, `cast`/`xx`, then a postfix loop
  (`.member`, `.*`, `[i]`, `(args)`, `.{..}`, `.[..]`, `.(T)`).
- Statements are one shared grammar for files, struct bodies and blocks (`stmt.rs`). Declarations
  (`decl.rs`) are recognised by lookahead: `name {, name} :`, `::` or `:=`.
- Terminators: a statement needs `;` unless the previous token closed a brace body, a `#string`
  here-string, or `{} #flags` (`Parser::ends_block`). `@notes` after a statement attach to it.

### Lookahead decisions worth knowing

| Question | Rule |
| --- | --- |
| `(` starts a procedure header or a parenthesized expression? | `paren_starts_header`: header if the matching `)` is followed by `->`/`=>` or a header directive (`#c_call`, ...), or the interior is empty / starts like a parameter (`name:`, `$T`, `using`, `..`, a top-level comma). |
| Struct literal or block? | `.{` is a single token, so `T.{..}` is always a literal and `cond {` is always a block. A bare `{ a = 1 }` in expression position is a literal only if it has no `;`. |
| `if x == {` | `peek_binary_op` refuses `==` when `{` or `#complete {` follows, so the `if` parser sees a switch. |
| `[2]int.[1, 2]` | The array element type is parsed without `.{`/`.[` postfix, so the literal applies to the whole array type. |
| Directive flags (`#run,stallable`) | `parse_directive_flags`: a comma hugging the directive and a following identifier. A few known flags (`distinct`, `file`, ...) may be spaced. |
| Return list commas | Inside argument/parameter lists (`in_list`) a comma ends a return type instead of adding a return value. |
| Mixed declarations | `a=, b := f()` and `a:, b = f()` produce a `Decl` with `existing` set per name. |

## How to change it

- New expression-level directive: add an arm in `directive.rs::parse_directive_expr` and an
  `ExprKind` variant in `ast.rs`.
- New statement-level directive: add an arm in `directive_stmt.rs::parse_directive_stmt`.
- New procedure flag: `procedure.rs::apply_directive_flag` (and a `ProcFlags` field, or add the
  name to `OTHER_FLAGS` to keep it by name only).
- New declaration flag: `DECL_FLAGS` in `decl.rs`. Flags on the next line are not consumed (they
  start the next statement).
- Gotchas: `continues_after_block` stops `(`, `[`, `*`, `-`, `+`, `<<` and `.` at the start of a
  line right after `}` so `Foo :: struct {}` followed by `*p = 1;` stays two statements.
  `AstId::fresh()` counters are per thread.

## Configuration

repo) and `JAIC_CORPUS_STAGE=lex|parse` configure the corpus sweep:

```sh
JAIC_CORPUS_ROOT=/path/to/jai JAIC_CORPUS_STAGE=parse \
  cargo test -p jaic --test corpus -- --ignored --nocapture
```

## Known non-parsing inputs

- `26.5_if.jai` (two copies in the book corpus): `#ifx c then a; else b` has a stray `;`.
- `focus-editor--focus/modules/uniform/program.jai`: a U+00A0 no-break space between tokens.

## Dependencies

Only `jaic::lexer`, `jaic::ast`, `jaic::intern` and `jaic::source`; no external crates.
