# `@Interpolates`: inline holes in string literals

This is a `jaic` extension. Official Jai has no such note.

## What it is

A procedure or `#expand` macro noted `@Interpolates("mode")` lets callers write expressions inside its format string: `f(db, "id = %user_id")`. The compiler turns that into the positional call `f(db, "id = %", user_id)` before type checking. The editor treats `user_id` as code: hover, go to definition, references, rename and completion all work inside the string.

```jai
describe :: (format: string, args: ..Any) -> string { /* ... */ } @Interpolates("plain")

query :: ($sql: string, args: ..Any) -> Rows #expand { /* ... */ } @Interpolates("sql")

rows := query("select * from users where id = %user_id and age > %(min_age + 1)");
```

## How it works

**Which parameter.** The one `@PrintLike` uses: a `string` parameter directly before a variadic `..Any`. It may be a constant parameter (`$sql: string`). The note is read from the procedure, on its header or after its body, like `@PrintLike`.

**Which calls.** `check_call` resolves the callee first (`sema/calls/mod.rs`), then `expand_interpolation` (`sema/interpolate.rs`) looks at the argument for that parameter. It acts only on a string literal or a `#string` block. Any other argument is left as it is, with no error. A literal with no holes is left alone too, so positional `%` calls keep working.

**Holes.** The scanner (`interpolate.rs`, shared with the language server) finds, in the literal's value:

| Form | Expression |
| --- | --- |
| `%name`, `%name.field.sub` | the name and its `.field` chain |
| `%array[i]` | `.field` and `[expr]` chains, in any order after the name |
| `%(any expression)` | what is inside the parentheses |

`%%` stays `%%` in the rewritten string; the callee decides what it means. A `%` followed by a space, the end of the string, a digit or any other character stays a plain `%`, so SQL `a % b` works. `a %b` is a hole: write `a %% b` or keep the space.

**Modes.** The note's argument picks what the scanner skips. The grammar is fixed in the compiler.

- `"plain"` skips nothing.
- `"sql"` skips `'...'` (with `''`), `E'...'` (with backslash escapes), `$tag$...$tag$`, `"..."`, `--` line comments and nested `/* */` comments. A `%` there is literal.

**Spans.** Each hole is lexed and parsed as an expression whose token spans point into the string in the source file. Escapes in a quoted literal are handled by a mapping from value offsets to source offsets (`lexer/value_map.rs`); `#string` blocks map one to one, except `\r` (dropped) and `,cr` (one `\n` is two bytes). So an undefined name or a type error in a hole is reported at the name inside the string, and rename edits the string text.

**Rewriting.** The string argument becomes the value with each hole replaced by `%`. The hole expressions become unnamed arguments, inserted right after it. If the call already passes positional arguments after the string (or a spread) and the string has holes, that is the error "a mix of inline holes and positional arguments". Named arguments after the string are fine.

**Overloads.** The rewrite happens when any candidate carries the note, and the candidates that carry it must agree on the parameter and the mode; if they disagree the call is left alone. Candidates without the note then receive the rewritten call and accept or reject it like any other.

## How to change it

- New hole forms or skip rules: `interpolate.rs` only. Its unit tests describe the grammar. The editor reads the same functions.
- The rewrite and the spans: `sema/interpolate.rs`. The expression parser entry is `parser::parse_expression`.
- A new mode: add it to `interpolate::Mode` and `Mode::from_name`, and to the table above.
- Editor behavior: `jai-language-server/src/interpolation.rs` finds the holes of the literals the compiler recorded (`Compiler::ide_interpolations`) and answers for them. `word` (hover, rename, definition), `semantic_tokens` and `completion` ask it.
- Gotchas: `check_call` runs every time a macro or polymorphic body is checked again, so the rewrite is repeated; it is pure, and it records the editor fact once per literal. The rewritten literal keeps the original span (`pr.interpolated`), so `@PrintLike` warnings do not offer fixes that edit the original text.

## Configuration

None. The note's argument is the only setting. Completion and highlighting need only the note, not compile-time execution, so they work in untrusted workspaces.

## Dependencies

The lexer's `string_value_map`, the parser's `parse_expression`, the `@PrintLike` parameter finder (`format_string_param`) and, for the editor, the compile's recorded facts (`IdeFacts::interpolations`). The Postgres client's `query` macro is the first user: see [the design](../postgres/design.md#inline-parameters).

## Limits

- The editor needs a session with an environment (a compile); without one holes are plain string text.
- `jailint` and `jaifmt` read syntax only, so they see a hole as string text.
