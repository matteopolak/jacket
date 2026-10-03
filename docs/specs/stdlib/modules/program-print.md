# Program_Print


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `get_plugin :: () -> *Metaprogram_Plugin`
- `print_procedure_bodies := true`
- `program_print_receive_message :: (p: *Metaprogram_Plugin, message: *Message)`
- `program_print_finish :: (p: *Metaprogram_Plugin)`
- `print_declaration :: (decl: *Code_Declaration) -> string`
- `print_string_literal :: (builder: *String_Builder, s: string)`
- `print_expression :: (builder: *String_Builder, expression: *Code_Node, skip_parens := false) -> printed: bool`
- `print_declaration :: (builder: *String_Builder, decl: *Code_Declaration, print_using := true, compound: *Code_Compound_Declaration = null, print_type := true) -> bool`


## Behavioral contract

`print_string_literal` appends a double-quoted Jai string literal to the builder. The public expression-printing example shows a line feed rendered as `\n`; the source literal grammar requires embedded quotes and backslashes to be escaped so the printed spelling preserves the value. The examples do not establish escape spellings for other control bytes.

`Program_Print` is a metaprogram plugin that receives compiler messages, groups declarations by source file, and emits a textual representation of parsed declarations. `print_expression`/`print_declaration` return text or append it to a builder; `finish` writes a per-file `pp_<basename>.txt` output. The output is intended for inspection/code browsing, not as a round-trip-preserving formatter: unsupported expression forms can be reported as unprinted, and file output names are derived from basenames. Requires the compiler metaprogram API, filesystem write access, and string-builder/file utilities.
