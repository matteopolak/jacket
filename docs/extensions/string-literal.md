# String literal locations

`String_Literal.location_of_byte(literal, byte)` maps a byte of a string literal's value to its source location, so compile-time checks of a string (an SQL query, a pattern, a format string) can report the exact character. Usage is in the module's doc comment. When `ok` is false the location is where the literal starts, so it is always safe to report at.

## How it works

- Literal nodes carry the file and the start line and column. The module re-reads the file (through the language server's overlay in the editor, so unsaved text counts), finds the literal and decodes it the way the lexer does, recording the source offset of each output byte.
- Quoted literals handle every escape and line continuation; a byte from an escape maps to its backslash. Here-strings handle the `,cr` and `,\%` flags.
- The decoded text must equal the node's `_string`; otherwise (a string built at compile time, or a changed file) `ok` is false.

## How to change it

A new escape or here-string flag in the lexer (`crates/jaic-syntax/src/lexer/scan.rs`) needs the same case in `decode_quoted` or `decode_here`. Until then those literals report `ok == false` rather than a wrong place. Columns are bytes.
