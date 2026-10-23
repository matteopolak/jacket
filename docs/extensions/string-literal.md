# String literal locations (`Extensions/String_Literal`)

## What it is

`String_Literal.location_of_byte(literal, byte)` gives the source location of one byte of a string literal's value. Compile-time code that checks a string it was given (an SQL query, a pattern, a format string) can then report a problem at the exact character instead of at the whole literal.

```jai
String_Literal :: #import "Extensions/String_Literal";

check :: (code: Code) {
    literal := cast(*Code_Literal) compiler_get_nodes(code);
    bad := index_of_problem(literal._string);
    location, ok := String_Literal.location_of_byte(literal, bad);
    compiler_report("unexpected token", location);
}
```

`byte == value.count` names the end of the value: for a quoted literal, the closing quote. When `ok` is false the location is where the literal starts, so it is always safe to report at.

## How it works

Literal nodes from `compiler_get_nodes` carry the file (`enclosing_load`), plus `l0`/`c0` as 1-based line and byte column. The module:

1. Reads the file again with `read_entire_file`. In the editor this goes through the language server's overlay, so it reads the unsaved buffer.
2. Finds the literal's first byte.
3. Decodes the literal the way the lexer does (`crates/jaic/src/lexer/scan.rs`), recording for each output byte the offset of the source that produced it:
   - **Quoted literals.** Every escape is handled: `\n \r \t \0 \e \a \b \f \v \% \\ \" \'`, `\xHH`, `\dDDD`, `\uHHHH`, `\UHHHHHHHH`, and line continuations. A byte produced by an escape maps to its backslash.
   - **Here-strings.** `#string TAG`, with the `,cr` and `,\%` flags. `\r` is dropped, and with `,cr` a line end becomes `\r\n`.
4. Checks that the decoded text equals the node's `_string`. If it does not (a string built by compile-time code, or a file changed since it was parsed), `ok` is false.

## How to change it

- **A new escape or here-string flag in the lexer** needs the same case here (`decode_quoted`, `decode_here`). Until it is added, literals using it report `ok == false` rather than a wrong place, because of the check in step 4.
- **Columns** are byte columns, like `Source_Code_Location` everywhere in jaic. Diagnostics convert them for display.

## Configuration

None.

## Dependencies

- Uses `Compiler` (node types), `File` (`read_entire_file`), `String` and `Basic`.
- Test: `stdlib/Extensions/String_Literal/tests/locations.jai`.
