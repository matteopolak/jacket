# Source positions

`jaic::source::LineIndex` is the one place that turns a byte offset into a line and column, and back. The compiler, diagnostic renderer, `jailint`, language server and playground all use it. Nothing else should count newlines.

```rust
let index = LineIndex::new(text);
let (line, col) = index.line_col(text, offset);   // one-based, columns in characters
```

## How it works

- `LineIndex::new` makes one pass; every query after is a binary search. A line ends at `\n`; a `\r` before it isn't part of the line, so CRLF behaves like LF.
- Column flavours: `line_col` (characters, what the CLI prints), `line_byte_col` (bytes, what `#location` and debug info use), `utf16_position` / `offset_utf16` (the language-server protocol's unit).
- Offsets past the end clamp. Offsets inside a character move back to its start.
- Each `SourceFile` owns an index (`SourceFile::lines()`). For one-off questions without an index use `source::line_start` / `line_end`; don't write `text[..x].rfind('\n')`.
- `SourceFile::path` is a `SourcePath`: `File(PathBuf)` or `Virtual(label)` for text with no file (`#import,string`, `#insert` expansions). Virtual sources print as `<label>`; the brackets belong to `Display`, the only place a name becomes text.

## How to change it

- The renderer takes the index through `Label::lines` and `Fix::lines`. A caller rendering many reports from one big file (jailint) builds the index once and passes it, or rendering goes quadratic.
- Protocol wrapping lives in `jai-language-server/src/position.rs`; position logic itself belongs in `jaic`.
- Keep the UTF-16 conversions exact, including surrogate pairs.
