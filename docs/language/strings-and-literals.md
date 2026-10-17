# Strings and literals

## What it is

String values (a byte count and a data pointer), their literal forms (escapes, `#char`, here strings), C-string conversion and comparison.

## How it works

A `string` is 16 bytes: an `s64` count and a `*u8`. Contents are arbitrary bytes, including NULs and invalid UTF-8. `string_body` in `lexer.rs` decodes quoted strings.

```jai
s := "a\tb\x41\d066é\0z";     // s.count == 9
c := #char "A";                // 65
text :: #string END
    "quoted" \n verbatim
END
```

- Escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD`, `\uHHHH`, `\UHHHHHHHH` (UTF-8). Anything else is a lexer error.
- A one-byte string constant next to an integer is that byte: `c == "-"`, `c - "0"`, a `u8` argument.
- Here strings (`here_string`, `lexer.rs`): the body starts on the next line and ends at a line whose first non-blank text is the terminator word (a longer identifier does not count; a body line starting with the terminator ends it). Indentation is kept, line endings become `\n` (`#string,cr` gives `\r\n`), and `#string,\%` turns `\%` into byte 0x1f.
- The terminator tag has no compiler meaning, but editors highlight the body by it (`#string WGSL`, `GLSL`, `SQL`, `JSON`, `JAI`); see [the VS Code extension](../tools/vscode-extension.md#embedded-languages).
- `==` compares length and bytes. A string literal in a `*u8` context becomes a pointer to static NUL-terminated bytes; a `string` variable does not convert (use `.data`). `for c: "hey"` iterates `u8`.

## How to change it

Escapes and here strings are in `lexer.rs`. The literal-to-`*u8` rule is in `convert` (`sema/convert.rs`), which treats a `Value::String` constant specially. A one-character string is a byte only where the target is `u8`: `x: u8 = "a"`, `b[0] = "z"`, and `+`, `-`, `==` next to a `u8` operand. With `u16`, `int` and the other integers it is a type mismatch, and a longer string never converts to `u8`. It does not pass through `ifx` either (`c: u8 = ifx t then "b" else "a"` is an error). Tests: `tests/stdlib/one-char-string-as-byte.jai`, `crates/jaic/tests/sema_version_drift.rs`, `string-scan.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- A `string` is 16 bytes: an `s64` count and a `*u8` {#str.1}. Contents are arbitrary bytes, so embedded NULs and invalid UTF-8 are fine {#str.2}. `string_body` in `lexer.rs` decodes quoted strings.
- The literal has count 9 and exactly those bytes {#str.3}.
- Escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD` (three decimal digits), `\uHHHH` and `\UHHHHHHHH` (encoded as UTF-8) {#str.4}. Anything else is a lexer error {#str.5}.
- `#char "A"` is the byte value `65` {#str.6}. A one-byte string constant next to an integer is that byte too: `c == "-"`, `c - "0"`, a `u8` argument (`split(s, ".")`) and `cast(u8, "\u001F")` {#str.19}.
- The body starts on the line after the header and ends at a line whose first non-blank text is the terminator word {#str.7} (a longer identifier doesn't count) {#str.8}. Indentation is kept {#str.9}. Line endings become `\n` {#str.10}; `#string,cr` makes them `\r\n` {#str.11}; `#string,\%` turns `\%` into byte 0x1f for `print` format strings {#str.12}.
- `==` and `!=` compare length and bytes: `"x\0y" == "x\0z"` is false {#str.13}.
- A string literal in a `*u8` context becomes a pointer to static bytes with a NUL appended, which is how literals reach C {#str.14}; `inspect("A\0B")` sees `A`, `0`, `B` {#str.15}. A `string` variable doesn't convert to `*u8` {#str.16}; use `.data` or the library's C-string helpers.
- `advance(*s, n)` (Basic) drops `n` leading bytes from a string view {#str.17}. `for c: "hey"` iterates bytes as `u8` {#str.18}.
