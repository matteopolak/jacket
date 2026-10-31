# Strings and literals

A `string` is 16 bytes: an `s64` count and a `*u8` {#str.1}. Contents are arbitrary bytes, so embedded NULs and invalid UTF-8 are fine {#str.2}.

```jai
s := "a\tb\x41\d066é\0z";     // s.count == 9
c := #char "A";                // 65
text :: #string END
    "quoted" \n verbatim
END
```

The first literal has count 9 and exactly those bytes {#str.3}.

## Literals

- Escapes: `\n \r \t \0 \e \a \b \f \v \\ \" \'`, `\%` (byte 0x1f), `\xHH`, `\dDDD` (three decimal digits), `\uHHHH` and `\UHHHHHHHH` (UTF-8) {#str.4}. Anything else is a lexer error {#str.5}.
- `#char "A"` is the byte `65` {#str.6}. A one-byte string constant next to an integer is that byte too: `c == "-"`, `c - "0"`, a `u8` argument (`split(s, ".")`), `cast(u8, "\u001F")` {#str.19}. It applies only where the target is `u8`: with `u16`, `int` and the like it is a type mismatch, a longer string never converts, and it doesn't pass through `ifx`.
- Here strings: the body starts on the line after the header and ends at a line whose first non-blank text is the terminator word {#str.7}; a longer identifier doesn't count {#str.8}. Indentation is kept {#str.9}. Line endings become `\n` {#str.10}; `#string,cr` makes them `\r\n` {#str.11}; `#string,\%` turns `\%` into byte 0x1f for `print` format strings {#str.12}.
- The terminator tag has no compiler meaning, but editors highlight the body by it (`WGSL`, `GLSL`, `SQL`, `JSON`, `JAI`); see [the VS Code extension](../tools/vscode-extension.md#how-it-works).

## Strings as values

- `==` and `!=` compare length and bytes: `"x\0y" == "x\0z"` is false {#str.13}.
- A string literal in a `*u8` context becomes a pointer to static bytes with a NUL appended, which is how literals reach C {#str.14}; `inspect("A\0B")` sees `A`, `0`, `B` {#str.15}.
- A `string` variable doesn't convert to `*u8` {#str.16}; use `.data` or the library's C-string helpers.
- `advance(*s, n)` (Basic) drops `n` leading bytes from a string view {#str.17}. `for c: "hey"` iterates bytes as `u8` {#str.18}.

## How to change it

Escapes and here strings are in `lexer.rs` (`string_body`, `here_string`). The literal-to-`*u8` rule is in `sema/convert.rs`, which treats a `Value::String` constant specially. Tests: `tests/compiler/regressions/one-char-string-as-byte.jai`.
