# Strings and literals

## What it is

String values (a byte count and a data pointer), their literal forms (escapes, `#char`, here strings), C-string conversion, and comparison.

## How it works

A `string` is 16 bytes: an `s64` count and a `*u8` {#str.1}. Contents are arbitrary bytes, so embedded NULs and invalid UTF-8 are fine {#str.2}. `string_body` in `lexer.rs` decodes quoted strings.

```jai
s := "a\tb\x41\d066é\0z";
// s.count == 9; bytes 97 9 98 65 66 195 169 0 122
```

The literal has count 9 and exactly those bytes {#str.3}.

Escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD` (three decimal digits), `\uHHHH` and `\UHHHHHHHH` (encoded as UTF-8) {#str.4}. Anything else is a lexer error {#str.5}.

`#char "A"` is the byte value `65` {#str.6}. A one-byte string constant next to an integer is that byte too: `c == "-"`, `c - "0"`, a `u8` argument (`split(s, ".")`) and `cast(u8, "\u001F")` {#str.19}.

Here strings (`here_string` in `lexer.rs`):

```jai
text :: #string END
    "quoted" \n verbatim
END
```

The body starts on the line after the header and ends at a line whose first non-blank text is the terminator word {#str.7} (a longer identifier doesn't count) {#str.8}. Indentation is kept {#str.9}. Line endings become `\n` {#str.10}; `#string,cr` makes them `\r\n` {#str.11}; `#string,\%` turns `\%` into byte 0x1f for `print` format strings {#str.12}.

Gotcha: the terminator is matched after stripping indentation, so a body line that starts with the terminator word ends the string.

Editor convention: the terminator can say what the body is. The VS Code extension and the playground highlight the body of `#string WGSL`, `#string GLSL`, `#string SQL`, `#string JSON`, `#string JAI` and other language names as that language (ignoring case; the full table is in [the VS Code extension's docs](../tools/vscode-extension.md#embedded-languages)), and `END`, `DONE` and other words as a plain string. The compiler gives the tag no meaning beyond ending the string, so pick the language's name when the body is code:

```jai
SHADER :: #string WGSL
@fragment fn fs() -> @location(0) vec4f { return vec4f(1.0); }
WGSL
```

`==` and `!=` compare length and bytes: `"x\0y" == "x\0z"` is false {#str.13}.

A string literal in a `*u8` context becomes a pointer to static bytes with a NUL appended, which is how literals reach C {#str.14}; `inspect("A\0B")` sees `A`, `0`, `B` {#str.15}. A `string` variable doesn't convert to `*u8` {#str.16}; use `.data` or the library's C-string helpers.

`advance(*s, n)` (Basic) drops `n` leading bytes from a string view {#str.17}. `for c: "hey"` iterates bytes as `u8` {#str.18}.

## How to change it

Escapes and here strings are in `lexer.rs`. The rule that turns literals into `*u8` is in `convert` in `sema/convert.rs`, which treats a `Value::String` constant specially. Tests: `tests/stdlib/one-char-string-as-byte.jai`, `string-scan.jai`.

## Dependencies

`lexer.rs`, `sema/convert.rs`.
