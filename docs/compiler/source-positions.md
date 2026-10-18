# Source positions

## What it is

`jaic::source::LineIndex` is the one place that turns a byte offset into a line and column (and back). The compiler, the diagnostic renderer, `jailint`, the language server and the playground all use it; nothing else should count newlines.

## How it works

`LineIndex::new(text)` makes one pass and records where every line starts and ends. Every query after that is a binary search. A line ends at `\n`; a `\r` in front of it (or ending the text) is not part of the line, so CRLF files behave like LF files.

| Query | Meaning |
| --- | --- |
| `line_of(off)`, `line_start(off)`, `line_end(off)` | zero-based line holding `off`, and its byte range without the line ending |
| `line_range(line)`, `lines()` | the range of a zero-based line, or of every line |
| `line_col(text, off)` | one-based line and column, the column counted in characters (what the CLI prints) |
| `line_byte_col(off)` | one-based line and column, the column counted in bytes |
| `utf16_position(text, off)` | zero-based line and UTF-16 column, `None` inside a character or past the end |
| `offset_utf16(text, line, col)` | the inverse; a column past the line end names the end, a column inside a surrogate pair is `Utf16Error::SplitsPair` |

Each `SourceFile` owns its index (`SourceFile::lines()`); `SourceFile::line_col` keeps the byte column the compiler's own locations (`#location`, debug info) use. Offsets past the end are clamped; offsets inside a character move back to it for character columns.

For a caller that asks once and has no index, `source::line_start(text, off)` and `source::line_end(text, off)` do the same on a `&str` in time proportional to the line, not the file. Do not write `text[..x].rfind('\n')`.

## How to change it

- The renderer takes the index through `Label::lines` and `Fix::lines`. `Diagnostic::report` fills them from the `SourceFile`. A caller that renders many reports from one large file (jailint) builds a `LineIndex` once and passes it; leaving it `None` builds one per `render`, which is fine for a single report but makes thousands of reports quadratic. There is no global cache.
- `jai-language-server/src/position.rs` wraps the index with the protocol's `Position` and error messages. Position logic belongs in `jaic`, not there.
- Keep `utf16_position` and `offset_utf16` exact: the protocol's unit is UTF-16 code units. The unit tests include a randomized comparison against the language server's original implementation.
- The playground reports character columns, like the CLI.

## Configuration

None.

## Dependencies

`jaic::source` only. The renderer (`jaic::render`) depends on it, not the other way round.
