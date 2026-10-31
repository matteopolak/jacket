#!/usr/bin/env python3
"""Emit the runtime lexer's public data declarations from the reviewed API page."""

import re

from generators.jai_codegen import emit_generated, generator_parser
from toolkit.repo import ROOT

OUTPUTS = [
    "stdlib/Jai_Lexer/module.jai",
    "stdlib/internal/jai-lexer/keywords.jai",
]
FORMATTED = True

MODULE_DOC = """Turns Jai source text into tokens, with their types, values and positions.

Set a [Lexer]'s input with [set_input_from_string] or [set_input_from_file], look at upcoming tokens with [peek_next_token] and [peek_token], and consume them with [eat_token]. The end of the input is a token of type `END_OF_INPUT`.

```jai
lexer: Lexer;
set_input_from_string(*lexer, "orchid 23");

token := peek_next_token(*lexer);
assert(token.type == .IDENT && token.ident_value.name == "orchid");
eat_token(*lexer);

token = peek_next_token(*lexer);
assert(token.type == .NUMBER && token.integer_value == 23);
```"""

DOCS = {
    "Hex_Digit_Type": "Which hex escape a string is reading: `\\x` (two digits), `\\u` (four) or `\\U` (eight).",
    "Note_Flags": "Flags on a `@note`.",
    "Token": "One token: its `type`, where it starts (`l0`, `c0`, line and column) and ends (`l1`, `c1`), and its value. Read `ident_value` for an identifier, `integer_value` or `float64_value` for a number (check `value_flags`), `string_value` for a string.",
    "Token_Type": "What kind of token this is. Single-character tokens use their character as the value (`#char \"+\"`); everything else is named.",
    "Value_Flags": "How a string or number token was written: as a here-string, in hex or binary, as a float, a float that needs 64-bit precision, or an integer that overflowed.",
    "Lexer": "The state of a lex in progress: the input, the position, and the tokens read ahead. Start from a zeroed `Lexer` and call [set_input_from_string] or [set_input_from_file].",
}


def documented(name, text):
    doc = DOCS.get(name)
    return "".join("// " + line + "\n" for line in doc.splitlines()) + text if doc else text


def main():
    args = generator_parser(__doc__).parse_args()
    source = (ROOT / "docs/specs/stdlib/modules/jai-lexer.md").read_text()
    inventory = source.split("## Public type members")[0]
    constants = re.findall(r"^- `([A-Z_]+) :: (\d+|false)`$", inventory, re.M)
    chunks = ["\n".join(("// " + line).rstrip() for line in MODULE_DOC.splitlines()) + "\n"]
    chunks += [name + " :: " + value + ";" for name, value in sorted(constants)]
    chunks.append("Table_String :: #type,isa string;")
    backing = {"Token_Type": "enum s16", "Note_Flags": "enum_flags u8", "Value_Flags": "enum_flags u32", "Hex_Digit_Type": "enum u8", "Lexer": "struct"}
    sections = re.findall(r"^#### `([^`]+)`[^\n]*\n(.*?)(?=^#### |^## |\Z)", source, re.M | re.S)
    # Alphabetic lookup within vocabulary, token records, then scanner state.
    for name, body in sorted(sections, key=lambda item: (item[0] == "Lexer", item[0])):
        block = re.search(r"```jai\n(.*?)```", body, re.S)
        if block:
            chunks.append(documented(name, re.sub(r"\\\s*_", "_", block[1]).strip()))
            continue
        fields = re.findall(r"^- `(.*)`$", body, re.M)
        lines = [name + " :: " + backing[name] + " {"]
        for field in fields:
            if name == "Lexer" and "atom_pool:" in field:
                field = "atom_pool: P.Pool"
            lines.append("    " + field.rstrip(";") + ";")
        lines.append("}")
        chunks.append(documented(name, "\n".join(lines)))
    functions = list(dict.fromkeys(re.findall(r"^- `([a-z_]+) ::", inventory, re.M)))
    chunks += [name + " :: Impl." + name + ";" for name in sorted(functions)]
    chunks.append('#scope_file\nP :: #import "Pool";\nImpl :: #import "internal/jai-lexer";')
    out = ROOT / "stdlib/Jai_Lexer/module.jai"
    out.parent.mkdir(parents=True, exist_ok=True)
    emit_generated(out, "\n\n".join(chunks) + "\n", args.formatter, args.check)
    keyword_names = re.findall(r"^- `(KEYWORD_[A-Z_]+)\s*::", source, re.M)
    lines = ["// Sets an identifier token's type to the keyword it spells, if any, and returns the type.", "//", "// ```jai", "// lexer: Lexer;", '// set_input_from_string(*lexer, "struct");', "// token := peek_next_token(*lexer);", "// assert(check_for_keyword(token) == .KEYWORD_STRUCT);", "// ```", "check_for_keyword :: (token: *J.Token) -> J.Token_Type {"]
    for name in sorted(keyword_names):
        lines.append('    if token.ident_value.name == "' + name.removeprefix("KEYWORD_").lower() + '" { token.type = .' + name + '; return token.type; }')
    lines += ["    return token.type;", "}", "#scope_module", "punctuation :: (lexer: *J.Lexer) -> (type: J.Token_Type, width: s64) {"]
    operators = {
        "<<<=": "ROTATE_LEFT_EQUALS", ">>>=": "ROTATE_RIGHT_EQUALS",
        "<<<": "ROTATE_LEFT", ">>>": "ROTATE_RIGHT", "===": "TRIPLE_EQUALS",
        "<<=": "SHIFT_LEFT_EQUALS", ">>=": "SHIFT_RIGHT_EQUALS", "&&=": "LOGICAL_AND_EQUALS", "||=": "LOGICAL_OR_EQUALS",
        "+=": "PLUSEQUALS", "-=": "MINUSEQUALS", "*=": "TIMESEQUALS", "/=": "DIVEQUALS", "%=": "MODEQUALS",
        "==": "ISEQUAL", "!=": "ISNOTEQUAL", "&&": "LOGICAL_AND", "||": "LOGICAL_OR", "<=": "LESSEQUALS", ">=": "GREATEREQUALS",
        "->": "RIGHT_ARROW", "..": "DOUBLE_DOT", "$$": "DOUBLE_DOLLAR", "<<": "SHIFT_LEFT", ">>": "SHIFT_RIGHT",
        "&=": "BITWISE_AND_EQUALS", "|=": "BITWISE_OR_EQUALS", "^=": "BITWISE_XOR_EQUALS", ".*": "POSTFIX_DEREFERENCE",
        "---": "TRIPLE_MINUS", "--": "DOUBLE_MINUS", ",,": "DOUBLE_COMMA", ".{": "BEGIN_STRUCT_LITERAL", ".[": "BEGIN_ARRAY_LITERAL", "=>": "QUICK_LAMBDA",
    }
    for spelling, name in sorted(operators.items(), key=lambda item: (-len(item[0]), item[0])):
        lines.append('    if matches_text(lexer, "' + spelling + '") return .' + name + ', ' + str(len(spelling)) + ';')
    lines += ["    return cast(J.Token_Type) look(lexer, 0), 1;", "}", '#scope_file', 'J :: #import "Jai_Lexer";']
    helper = ROOT / "stdlib/internal/jai-lexer/keywords.jai"
    helper.parent.mkdir(parents=True, exist_ok=True)
    emit_generated(helper, "\n".join(lines) + "\n", args.formatter, args.check)
    print(f"{'Verified' if args.check else 'Generated'} {out.relative_to(ROOT)} and {helper.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
