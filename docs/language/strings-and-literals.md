# Strings and literals

## What it is

String values (a byte count plus a data pointer), their literal forms (escapes, `#char`, here strings), C-string conversion, and string comparison.

## How it works

A `string` is 16 bytes: an `s64` count and a `*u8`. Contents are arbitrary bytes, so embedded NULs and invalid UTF-8 are fine. `string_body` in `crates/jaic/src/lexer.rs` decodes quoted strings.

```jai
s := "a\tb\x41\d066é\0z";
// s.count == 9; bytes 97 9 98 65 66 195 169 0 122
```

Supported escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD` (three decimal digits), `\uHHHH` and `\UHHHHHHHH` (encoded as UTF-8). An unknown escape is a lexer error.

`#char "A"` yields the byte value `65`.

Here strings:

```jai
text :: #string END
    "quoted" \n verbatim
END
```

The body starts on the line after the header and runs to a line whose first non-blank text is the terminator word (a longer identifier does not terminate). Leading indentation of body lines is kept. Line endings normalize to `\n`; `#string,cr` makes them `\r\n`; `#string,\%` turns `\%` into byte 0x1f (used by `print` format strings). See `here_string` in `lexer.rs`.

`==` and `!=` on strings compare length and bytes: `"x\0y" == "x\0z"` is false and `"abc" != "ab\0"` is true.

A string literal in a `*u8` context becomes a pointer to static bytes with one extra NUL appended, which is how literals reach C functions; `inspect("A\0B")` sees `A`, `0`, `B`. A `string` variable does not convert to `*u8`; take `.data` or use the library C-string helpers.

`advance(*s, n)` (Basic) drops `n` leading bytes from a string view; `for c: "hey"` iterates bytes as `u8` values.

## How to change it

Add or change escapes in `string_body`, and keep `#char` handling and here-string normalization in the same file. The expected-type rule that turns literals into `*u8` is in the conversion code (`convert` in `sema/convert.rs`, which treats a `Value::String` constant specially). Tests: `tests/stdlib/one-char-string-as-byte.jai` and `tests/stdlib/string-scan.jai`.

Gotcha: here-string terminators are matched after stripping indentation, so a body line that begins with the terminator word ends the string.

## Configuration

None.

## Dependencies

`crates/jaic/src/lexer.rs`, `sema/convert.rs`, and the `Basic` module for `print` and `advance`.
