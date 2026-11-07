#scope_module

proto_Token_Kind :: enum {
    END;
    IDENT;
    NUMBER;
    STRING;
    SYMBOL;
}

proto_Token :: struct {
    kind: proto_Token_Kind;
    text: string;
    line: s64;
    column: s64;
    path: string;
}

proto_Schema :: struct {
    text: string;
    options: Generate_Options;
    tokens: [..] proto_Token;
    pos: s64;
    error: Schema_Error;
    package: string;
    roots: [..] *proto_Node;
    nodes: [..] *proto_Node;
    services: [..] *proto_Service;
    imports: [..] proto_Import;
    sources: [..] *proto_Source;
    source: *proto_Source;
    names: proto_symbols.Map(string, *proto_Node);
    jai_names: proto_symbols.Map(string, *proto_Node);
    source_paths: proto_symbols.Map(string, *proto_Source);
    // Import lookups: by folder-relative candidate, and by name in the import roots.
    nearby_paths: proto_symbols.Map(string, string);
    import_roots: proto_symbols.Map(string, string);
}

proto_schema_fail :: (s: *proto_Schema, token: proto_Token, message: string) {
    if s.error.message.count return;
    path := token.path;
    if !path.count path = s.options.path;
    s.error = .{ message = message, path = path, line = token.line, column = token.column };
}

proto_letter :: (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_";
}

proto_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

proto_hex_digit :: (c: u8) -> s64 {
    if proto_digit(c) return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;
    return -1;
}

// Bytes that continue an identifier: letters, digits and '_'.
proto_WORD_BYTES :: #run -> [256] bool {
    result: [256] bool;
    for c: 0..255 result[c] = proto_letter(cast(u8) c) || proto_digit(cast(u8) c);
    return result;
};

// Whether the 8 bytes at data hold either byte, tested a word at a time (comments are most of a schema).
proto_word_has :: (data: *u8, a: u8, b: u8) -> bool {
    ONES: u64 : 0x0101_0101_0101_0101;
    HIGHS: u64 : 0x8080_8080_8080_8080;
    word: u64;

    memcpy(*word, data, 8);
    x := word ^ (ONES * a);
    y := word ^ (ONES * b);

    return ((x - ONES) & ~x & HIGHS) | ((y - ONES) & ~y & HIGHS) != 0;
}

proto_lex :: (s: *proto_Schema) { // jailint: allow(cognitive_complexity)
    i: s64;
    line: s64 = 1;
    // Columns count bytes from the start of the line.
    line_start: s64;
    // Unchecked reads: every index below is compared with size first.
    data := s.text.data;
    size := s.text.count;
    words := proto_WORD_BYTES.data;

    while i < size && !s.error.message.count {
        c := data[i];

        if c == 32 || c == 9 || c == 13 || c == 10 {
            i += 1;

            if c == 10 {
                line += 1;
                line_start = i;
            }

            continue;
        }

        start := i;
        tok := proto_Token.{ line = line, column = start - line_start + 1, path = s.options.path };

        if c == #char "/" && i + 1 < size && data[i + 1] == #char "/" {
            while i + 8 <= size && !proto_word_has(data + i, 10, 10) i += 8;
            while i < size && data[i] != 10 i += 1;
            continue;
        }

        if c == #char "/" && i + 1 < size && data[i + 1] == #char "*" {
            i += 2;

            while i + 1 < size && !(data[i] == #char "*" && data[i + 1] == #char "/") {
                if i + 8 <= size && !proto_word_has(data + i, #char "*", 10) {
                    i += 8;
                    continue;
                }

                if data[i] == 10 {
                    line += 1;
                    line_start = i + 1;
                }

                i += 1;
            }

            if i + 1 >= size {
                proto_schema_fail(s, tok, "unterminated comment");
                break;
            }

            i += 2;
            continue;
        }

        if proto_letter(c) {
            tok.kind = .IDENT;
            while i < size && words[data[i]] i += 1;
            tok.text = .{ i - start, data + start };
        } else if proto_digit(c) {
            tok.kind = .NUMBER;

            while i < s.text.count {
                byte := s.text[i];
                exponent_sign := (byte == #char "+" || byte == #char "-") && i > start && (s.text[i - 1] == #char "e" || s.text[i - 1] == #char "E");
                if !proto_letter(byte) && !proto_digit(byte) && byte != #char "." && !exponent_sign break;
                i += 1;
            }

            tok.text = slice(s.text, start, i - start);
        } else if c == 34 || c == 39 {
            tok.kind = .STRING;
            i += 1;

            // Literals without escapes are slices of the source.
            end := i;
            while end < size && data[end] != c && data[end] != 92 && data[end] != 10 && data[end] != 13 end += 1;

            if end < size && data[end] == c {
                tok.text = .{ end - i, data + i };
                i = end + 1;
                array_add(*s.tokens, tok);
                continue;
            }

            b: String_Builder;

            while i < s.text.count && s.text[i] != c {
                byte := s.text[i];

                i += 1;

                if byte == 10 || byte == 13 {
                    proto_schema_fail(s, tok, "newline in string literal");
                    break;
                }

                if byte == 92 {
                    if i >= s.text.count break;

                    byte = s.text[i];
                    i += 1;

                    if byte == #char "n" byte = 10;
                    else if byte == #char "r" byte = 13;
                    else if byte == #char "t" byte = 9;
                    else if byte == #char "a" byte = 7;
                    else if byte == #char "b" byte = 8;
                    else if byte == #char "f" byte = 12;
                    else if byte == #char "v" byte = 11;
                    else if byte == #char "x" || byte == #char "X" {
                        value: s64;
                        count: s64;

                        while count < 2 && i < s.text.count && proto_hex_digit(s.text[i]) >= 0 {
                            value = value * 16 + proto_hex_digit(s.text[i]);
                            i += 1;
                            count += 1;
                        }

                        if !count proto_schema_fail(s, tok, "expected hex escape digits");

                        byte = cast(u8) value;
                    } else if byte >= #char "0" && byte <= #char "7" {
                        value := cast(s64) byte - #char "0";
                        count := 1;

                        while count < 3 && i < s.text.count && s.text[i] >= #char "0" && s.text[i] <= #char "7" {
                            value = value * 8 + s.text[i] - #char "0";
                            i += 1;
                            count += 1;
                        }

                        if value > 255 proto_schema_fail(s, tok, "octal escape exceeds one byte");

                        byte = cast(u8) value;
                    } else if byte != 92 && byte != 34 && byte != 39 && byte != #char "?" proto_schema_fail(s, tok, "invalid string escape");
                }

                append(*b, string.{ 1, *byte });
            }

            if i >= s.text.count proto_schema_fail(s, tok, "unterminated string literal");
            else i += 1;

            tok.text = builder_to_string(*b);
        } else {
            tok.kind = .SYMBOL;
            tok.text = slice(s.text, i, 1);
            i += 1;
        }

        array_add(*s.tokens, tok);
    }

    array_add(*s.tokens, .{ kind = .END, line = line, column = i - line_start + 1, path = s.options.path });
}

proto_peek :: (s: *proto_Schema) -> proto_Token {
    return s.tokens[s.pos];
}

proto_take :: (s: *proto_Schema) -> proto_Token {
    result := proto_peek(s);
    if result.kind != .END s.pos += 1;
    return result;
}

proto_eat :: (s: *proto_Schema, text: string) -> bool {
    token := *s.tokens[s.pos];
    if token.kind == .END || token.text != text return false;
    s.pos += 1;
    return true;
}

proto_expect :: (s: *proto_Schema, text: string) -> bool {
    if proto_eat(s, text) return true;
    proto_schema_fail(s, proto_peek(s), proto_schema_format("expected '%'", text));
    return false;
}

proto_identifier :: (s: *proto_Schema) -> proto_Token {
    token := proto_take(s);
    if token.kind != .IDENT proto_schema_fail(s, token, "expected identifier");
    return token;
}

proto_qualified :: (s: *proto_Schema) -> string {
    b: String_Builder;
    if proto_eat(s, ".") append(*b, ".");

    append(*b, proto_identifier(s).text);

    while proto_eat(s, ".") {
        append(*b, ".");
        append(*b, proto_identifier(s).text);
    }

    return builder_to_string(*b);
}

proto_number :: (s: *proto_Schema) -> s64 {
    negative := proto_eat(s, "-");
    token := proto_take(s);

    if token.kind != .NUMBER {
        proto_schema_fail(s, token, "expected integer");
        return 0;
    }

    value: s64;
    base: s64 = 10;
    start: s64;

    if token.text.count > 1 && token.text[0] == #char "0" {
        base = 8;

        if token.text[1] == #char "x" || token.text[1] == #char "X" {
            base = 16;
            start = 2;
        }
    }

    for i: start..token.text.count - 1 {
        part := proto_hex_digit(token.text[i]);
        if part < 0 || part >= base || value > (0x7fffffffffffffff - part) / base {
            proto_schema_fail(s, token, "integer out of range");
            return 0;
        }

        value = value * base + part;
    }

    if negative value = -value;

    return value;
}
