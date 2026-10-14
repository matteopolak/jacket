#!/usr/bin/env python3
"""Emit the runtime lexer's public data declarations from the reviewed API page."""
from pathlib import Path
import argparse
import re
from jai_codegen import DEFAULT_FORMATTER, emit_generated


ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--formatter", type=Path, default=DEFAULT_FORMATTER)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    source = (ROOT / "docs/specs/stdlib/modules/jai-lexer.md").read_text()
    inventory = source.split("## Public type members")[0]
    constants = re.findall(r"^- `([A-Z_]+) :: (\d+|false)`$", inventory, re.M)
    chunks = ["// Generated from the reviewed Jai_Lexer public contract.\n"]
    chunks += [name + " :: " + value + ";" for name, value in sorted(constants)]
    chunks.append("Table_String :: #type,isa string;")
    backing = {"Token_Type": "enum s16", "Note_Flags": "enum_flags u8", "Value_Flags": "enum_flags u32", "Hex_Digit_Type": "enum u8", "Lexer": "struct"}
    sections = re.findall(r"^#### `([^`]+)`[^\n]*\n(.*?)(?=^#### |^## |\Z)", source, re.M | re.S)
    # Alphabetic lookup within vocabulary, token records, then scanner state.
    for name, body in sorted(sections, key=lambda item: (item[0] == "Lexer", item[0])):
        block = re.search(r"```jai\n(.*?)```", body, re.S)
        if block:
            chunks.append(re.sub(r"\\\s*_", "_", block[1]).strip())
            continue
        fields = re.findall(r"^- `(.*)`$", body, re.M)
        lines = [name + " :: " + backing[name] + " {"]
        for field in fields:
            if name == "Lexer" and "atom_pool:" in field:
                field = "atom_pool: P.Pool"
            lines.append("    " + field.rstrip(";") + ";")
        lines.append("}")
        chunks.append("\n".join(lines))
    functions = list(dict.fromkeys(re.findall(r"^- `([a-z_]+) ::", inventory, re.M)))
    chunks += [name + " :: Impl." + name + ";" for name in sorted(functions)]
    chunks.append('#scope_file\nP :: #import "Pool";\nImpl :: #import "internal/jai-lexer";')
    out = ROOT / "stdlib/Jai_Lexer/module.jai"
    out.parent.mkdir(parents=True, exist_ok=True)
    emit_generated(out, "\n\n".join(chunks) + "\n", args.formatter, args.check)
    keyword_names = re.findall(r"^- `(KEYWORD_[A-Z_]+)\s*::", source, re.M)
    lines = ["// Generated keyword spellings and documented input operators.", "check_for_keyword :: (token: *J.Token) -> J.Token_Type {"]
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
    print(out)


if __name__ == "__main__":
    main()
