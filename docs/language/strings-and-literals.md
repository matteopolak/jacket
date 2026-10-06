# Strings and literals

## What it is

String values (a byte count and a data pointer), their literal forms (escapes, `#char`, here strings), C-string conversion, and comparison.

## How it works

A `string` is 16 bytes: an `s64` count and a `*u8`. Contents are arbitrary bytes, so embedded NULs and invalid UTF-8 are fine. `string_body` in `lexer.rs` decodes quoted strings.

```jai
s := "a\tb\x41\d066é\0z";
// s.count == 9; bytes 97 9 98 65 66 195 169 0 122
```

Escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD` (three decimal digits), `\uHHHH` and `\UHHHHHHHH` (encoded as UTF-8). Anything else is a lexer error.

`#char "A"` is the byte value `65`.

Here strings (`here_string` in `lexer.rs`):

```jai
text :: #string END
    "quoted" \n verbatim
END
```

The body starts on the line after the header and ends at a line whose first non-blank text is the terminator word (a longer identifier doesn't count). Indentation is kept. Line endings become `\n`; `#string,cr` makes them `\r\n`; `#string,\%` turns `\%` into byte 0x1f for `print` format strings.

Gotcha: the terminator is matched after stripping indentation, so a body line that starts with the terminator word ends the string.

`==` and `!=` compare length and bytes: `"x\0y" == "x\0z"` is false.

A string literal in a `*u8` context becomes a pointer to static bytes with a NUL appended, which is how literals reach C. A `string` variable doesn't convert to `*u8`; use `.data` or the library's C-string helpers.

`for c: "hey"` iterates bytes as `u8`.

## How to change it

Escapes and here strings are in `lexer.rs`. The rule that turns literals into `*u8` is in `convert` in `sema/convert.rs`, which treats a `Value::String` constant specially. Tests: `tests/stdlib/one-char-string-as-byte.jai`, `string-scan.jai`.

## Dependencies

`lexer.rs`, `sema/convert.rs`.
