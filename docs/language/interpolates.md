# `@Interpolates`: inline holes in string literals

A jaic extension. A procedure or `#expand` macro noted `@Interpolates("mode")` lets callers write expressions inside its format string: `f(db, "id = %user_id")`. The compiler turns that into the positional call `f(db, "id = %", user_id)` before type checking. The editor treats `user_id` as code, so hover, go to definition, references, rename and completion work inside the string.

```jai
describe :: (format: string, args: ..Any) -> string { /* ... */ } @Interpolates("plain")

query :: ($sql: string, args: ..Any) -> Rows #expand { /* ... */ } @Interpolates("sql")

rows := query("select * from users where id = %user_id and age > %(min_age + 1)");
```

## How it works

- The format parameter is the one `@PrintLike` uses: a `string` directly before a variadic `..Any`. It may be a constant parameter (`$sql: string`). The note goes on the header or after the body.
- Only a string literal or `#string` block argument is rewritten. Anything else is left alone, with no error, and so is a literal with no holes.
- Hole forms:

| Form | Expression |
| --- | --- |
| `%name`, `%name.field.sub` | the name and its `.field` chain |
| `%array[i]` | `.field` and `[expr]` chains, in any order after the name |
| `%(any expression)` | what is inside the parentheses |

- `%%` stays `%%`; the callee decides what it means. A `%` followed by a space, the end of the string, a digit or any other character stays a plain `%`, so SQL `a % b` works. `a %b` is a hole: write `a %% b` or keep the space.
- Modes pick what the scanner skips. `"plain"` skips nothing. `"sql"` skips `'...'` (with `''`), `E'...'`, `$tag$...$tag$`, `"..."`, `--` comments and nested `/* */` comments, where `%` is literal.
- Each hole is parsed as an expression whose spans point into the string in the source, so an undefined name or type error is reported at the name inside the string.
- The string becomes the value with each hole replaced by `%`, and the hole expressions are inserted as unnamed arguments right after it. Positional arguments after a string with holes are an error ("a mix of inline holes and positional arguments"); named arguments are fine.
- With overloads, the rewrite happens when any candidate carries the note and those candidates agree on the parameter and mode; otherwise the call is left alone.

## How to change it

- New hole forms or skip rules: `interpolate.rs` only; its unit tests describe the grammar and the editor reads the same functions.
- The rewrite and spans: `sema/interpolate.rs`. A new mode goes in `interpolate::Mode` and `Mode::from_name`, and in the docs above.
- Editor behaviour: `jai-language-server/src/interpolation.rs`.
- `check_call` reruns when a macro or polymorphic body is rechecked, so the rewrite repeats; it is pure. The rewritten literal keeps the original span, so `@PrintLike` warnings offer no fixes that edit the original text.

## Limits

- The editor needs a session with a compile; without one, holes are plain string text.
- `jailint` and `jaifmt` read syntax only, so they see a hole as string text.

The Postgres client's `query` macro is the first user: see [the design](../postgres/design.md#inline-parameters).
