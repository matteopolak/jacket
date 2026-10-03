peek_next_character :: (lexer: *J.Lexer) -> s16 {
    if lexer.input_cursor >= lexer.input.count return -1;
    return cast(s16) lexer.input[lexer.input_cursor];
}

eat_character :: (using lexer: *J.Lexer) {
    if input_cursor >= input.count return;

    c := input[input_cursor];

    input_cursor += 1;

    if c == 10 {
        current_line_number += 1;
        current_external_file_error_report_line_number = current_line_number;
        current_character_index = 1;
        total_lines_processed += 1;
    } else current_character_index += 1;
}

unwind_one_character :: (lexer: *J.Lexer) {
    if lexer.input_cursor <= 0 return;

    desired := lexer.input_cursor - 1;

    lexer.input_cursor = 0;
    lexer.current_line_number = 1;
    lexer.current_character_index = 1;
    lexer.total_lines_processed = 0;
    while lexer.input_cursor < desired eat_character(lexer);
}

starts_identifier :: inline (c: u8) -> bool {
    return (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || c == 95;
}

continues_identifier :: (c: s32) -> bool {
    return c >= 0 && c <= 255 && (starts_identifier(cast(u8) c) || (c >= 48 && c <= 57));
}

starts_number :: (c: s32) -> bool {
    return c >= 48 && c <= 57;
}

set_end_of_token :: (lexer: *J.Lexer, token: *J.Token) {
    if !token return;
    token.l1 = lexer.current_line_number;
    token.c1 = lexer.current_character_index;
}

cached_make_atom :: (lexer: *J.Lexer, name: string, token_to_fill: *J.Token) {
    if !lexer.atom_pool.block_allocator.proc P.set_allocators(*lexer.atom_pool, context.allocator);

    data := cast(*u8) P.get(*lexer.atom_pool, name.count + 1);
    if !data {
        report_parse_error(lexer, "out of memory while copying an identifier");
        token_to_fill.type = .ERROR;
        return;
    }

    for c, i: name data[i] = c;

    data[name.count] = 0;
    token_to_fill.ident_value.name = cast(J.Table_String) string.{ name.count, data };
    token_to_fill.ident_value.hash = 0; // The public contract leaves this hash unspecified.
}

get_hex_digit :: (lexer: *J.Lexer, type: J.Hex_Digit_Type) -> (value: u8, success: bool) {
    c := peek_next_character(lexer);
    value: s16 = -1;

    if c >= 48 && c <= 57 value = c - 48;
    else if c >= 65 && c <= 70 value = c - 65 + 10;
    else if c >= 97 && c <= 102 value = c - 97 + 10;

    if value < 0 {
        if type == .ASCII_8_BIT report_parse_error(lexer, "\\x escape requires two hexadecimal digits");
        else if type == .UNICODE_16 report_parse_error(lexer, "\\u escape requires four hexadecimal digits");
        else report_parse_error(lexer, "\\U escape requires eight hexadecimal digits");
        return 0, false;
    }

    eat_character(lexer);

    return cast(u8) value, true;
}

get_decimal_digit :: (lexer: *J.Lexer) -> s32 {
    c := peek_next_character(lexer);
    if c < 48 || c > 57 {
        report_parse_error(lexer, "\\d escape requires three decimal digits");
        return -1;
    }

    eat_character(lexer);

    return cast(s32) (c - 48);
}

report_parse_error :: (lexer: *J.Lexer, format: string, arguments: ..Any) {
    lexer.reported_error = true;
    diagnostic(lexer.current_line_number, lexer.current_character_index, format, ..arguments);
}

report_parse_error :: (lexer: *J.Lexer, token: *J.Token, format: string, arguments: ..Any) {
    lexer.reported_error = true;
    if token diagnostic(token.l0, token.c0, format, ..arguments);
    else diagnostic(lexer.current_line_number, lexer.current_character_index, format, ..arguments);
}

report_detail :: (lexer: *J.Lexer, format: string, arguments: ..Any) {
    diagnostic(lexer.current_line_number, lexer.current_character_index, format, ..arguments);
}

eat_until_newline :: (lexer: *J.Lexer) {
    while peek_next_character(lexer) >= 0 && peek_next_character(lexer) != 10 eat_character(lexer);
}

eat_input_due_to_block_comment :: (lexer: *J.Lexer) {
    // Entry is just after the opening /*. Jai block comments nest.
    depth: s64 = 1;

    while lexer.input_cursor < lexer.input.count {
        if look(lexer, 0) == 47 && look(lexer, 1) == 42 {
            eat_character(lexer);
            eat_character(lexer);
            depth += 1;
        } else if look(lexer, 0) == 42 && look(lexer, 1) == 47 {
            eat_character(lexer);
            eat_character(lexer);
            depth -= 1;
            if depth == 0 return;
        } else eat_character(lexer);
    }

    report_parse_error(lexer, "unterminated block comment");
}

#scope_module
look :: (lexer: *J.Lexer, ahead: s64) -> s32 {
    at := lexer.input_cursor + ahead;
    if at < 0 || at >= lexer.input.count return -1;
    return cast(s32) lexer.input[at];
}

diagnostic :: (line: s32, column: s32, format: string, args: ..Any) {
    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    Basic.print_to_builder(*builder, "%:%: ", line, column);
    Basic.print_to_builder(*builder, format, ..args);
    Basic.append(*builder, "\n");
    text := Basic.builder_to_string(*builder);

    write_string(text, to_standard_error = true);
    Basic.free(text);
    Basic.free_buffers(*builder);
}

#scope_file
J :: #import "Jai_Lexer";
Basic :: #import "Basic";
P :: #import "Pool";
