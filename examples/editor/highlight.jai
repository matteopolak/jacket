// Syntax colouring for Jai source, one line at a time: comments, strings, numbers, keywords,
// directives and the names declared with `::`. It is a lexical approximation (a block comment or a
// multi-line string that spans lines is coloured only on the line it starts), which is what a
// small editor needs; the language server would give exact tokens.

COMMENT_COLOR :: 0x6A9955;
STRING_COLOR :: 0xCE9178;
NUMBER_COLOR :: 0xB5CEA8;
KEYWORD_COLOR :: 0xC586C0;
DIRECTIVE_COLOR :: 0x9CDCFE;
TYPE_COLOR :: 0x4EC9B0;
DECLARATION_COLOR :: 0xDCDCAA;

KEYWORDS :: string.["if", "else", "then", "for", "while", "return", "break", "continue", "case", "defer", "using", "cast", "xx", "struct", "enum", "enum_flags", "union", "inline", "remove", "push_context", "true", "false", "null", "it", "it_index", "context", "ifx", "size_of", "type_of", "initializer_of", "#run", "#insert"];
TYPE_NAMES :: string.["int", "float", "float32", "float64", "bool", "string", "void", "u8", "u16", "u32", "u64", "s8", "s16", "s32", "s64", "Type", "Any", "Code"];

// The procedure the editor widget calls for each visible line.
highlight_jai :: (line: string, spans: *[..] Highlight_Span) {
    i := 0;

    while i < line.count {
        c := line.data[i];

        if c == #char "/" && i + 1 < line.count && line.data[i + 1] == #char "/" {
            array_add(spans, .{ i, line.count, hex(COMMENT_COLOR) });
            return;
        }

        if c == #char "\"" {
            end := i + 1;

            while end < line.count && line.data[end] != #char "\"" {
                if line.data[end] == #char "\\" end += 1;
                end += 1;
            }

            end = min(end + 1, line.count);
            array_add(spans, .{ i, end, hex(STRING_COLOR) });
            i = end;
            continue;
        }

        if number_char(c) || (c == #char "." && i + 1 < line.count && number_char(line.data[i + 1]) && (i == 0 || !is_word(line.data[i - 1]))) {
            end := i + 1;
            while end < line.count && (is_word(line.data[end]) || line.data[end] == #char ".") end += 1;

            array_add(spans, .{ i, end, hex(NUMBER_COLOR) });
            i = end;
            continue;
        }

        if c == #char "#" || is_word(c) {
            end := i + 1;
            while end < line.count && is_word(line.data[end]) end += 1;

            word := string.{ end - i, line.data + i };
            color := word_color(word, line, end);

            if color.w > 0 array_add(spans, .{ i, end, color });

            i = end;
            continue;
        }

        i += 1;
    }
}

#scope_file

number_char :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_word :: (c: u8) -> bool {
    return number_char(c) || (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" || c >= 0x80;
}

word_color :: (word: string, line: string, end: int) -> Color {
    for KEYWORDS if it == word return hex(KEYWORD_COLOR);
    for TYPE_NAMES if it == word return hex(TYPE_COLOR);

    if word.data[0] == #char "#" return hex(DIRECTIVE_COLOR);

    // `name :: ...` and `name := ...` declare something.
    rest := end;
    while rest < line.count && line.data[rest] == #char " " rest += 1;

    if rest + 1 < line.count && line.data[rest] == #char ":" && (line.data[rest + 1] == #char ":" || line.data[rest + 1] == #char "=") {
        return hex(DECLARATION_COLOR);
    }

    return .{};
}
