// Formatting part of a file: the top-level statements a byte range touches are formatted as a
// text of their own, and everything outside them is copied through untouched.
//
// A statement is whatever ends at a `;`, at the `}` that closes a declaration or block, or at a
// scope directive (`#scope_file`). The range snaps outward to whole statements, then to the lines
// they share with their neighbours, so a statement is never cut in half and two statements on one
// line are formatted together.

#scope_module

Statement_Range :: struct {
    start: s64; // byte range in the source, a trailing comment on the same line included
    end: s64;
    first_token: int;
}

format_range_text :: (src: string, from: s64, to: s64, config: Format_Config) -> Format_Result {
    result: Format_Result;

    if from < 0 || to < from || to > src.count {
        result.error = tprint("range %:% is outside the text (% bytes)", from, to, src.count);
        return result;
    }

    lexed := lex(src);

    defer array_free(lexed.tokens);

    if lexed.error.count {
        line, column := line_and_column(src, lexed.error_offset);
        result.error = tprint("%:%: %", line, column, lexed.error);
        return result;
    }

    statements := top_level_statements(lexed.tokens, src);

    defer array_free(statements);

    first := -1;
    last := -1;

    for statements {
        touches := ifx from == to then it.start <= from && from <= it.end else it.start < to && it.end > from;
        if !touches continue;

        if first < 0 first = it_index;

        last = it_index;
    }

    if first < 0 || in_off_region(lexed.tokens, src, statements[first].first_token) {
        result.output = copy_string(src);
        result.ok = true;
        return result;
    }

    while first > 0 && statements[first - 1].end > line_begin(src, statements[first].start) first -= 1;

    while last + 1 < statements.count && statements[last + 1].start < line_end(src, statements[last].end) last += 1;

    region_start := statements[first].start;
    region_end := statements[last].end;
    indent := region_start;

    while indent > 0 && (src[indent - 1] == #char " " || src[indent - 1] == #char "\t") indent -= 1;

    if indent == 0 || src[indent - 1] == #char "\n" region_start = indent;

    piece := slice(src, region_start, region_end - region_start);
    formatted := format_text(piece, config);

    if !formatted.ok {
        result.error = tprint("in the range: %", formatted.error);
        return result;
    }

    // The piece ends at a statement, so what the formatter adds after it goes.
    output := formatted.output;

    while output.count && (output[output.count - 1] == #char "\n" || output[output.count - 1] == #char "\r") output.count -= 1;

    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, slice(src, 0, region_start));

    if uses_crlf(src) && !uses_crlf(piece) {
        for *output {
            if it.* == #char "\n" append(*builder, "\r");
            append(*builder, slice(output, it_index, 1));
        }
    } else {
        append(*builder, output);
    }

    append(*builder, slice(src, region_end, src.count - region_end));

    result.output = builder_to_string(*builder);
    result.ok = true;

    return result;
} @TempAllocates

// The top-level statements of a lexed file, in order.
top_level_statements :: (tokens: [] Tok, src: string) -> [..] Statement_Range {
    out: [..] Statement_Range;
    depth := 0;
    begin := -1;
    last_code := -1;

    for tokens {
        if it.kind == .EOF break;

        if it.kind == .LINE_COMMENT || it.kind == .BLOCK_COMMENT continue;

        if begin < 0 begin = it_index;

        last_code = it_index;

        text := slice(src, it.start, it.end - it.start);
        ends := false;

        if it.kind == .PUNCT {
            if text == "{" || text == ".{" || text == "(" || text == "[" || text == ".[" depth += 1;

            if text == "}" || text == ")" || text == "]" depth -= 1;
        }

        // Unbalanced brackets are refused when the statements are formatted.
        if depth <= 0 {
            depth = 0;

            if it.kind == .PUNCT && text == ";" ends = true;

            if it.kind == .PUNCT && text == "}" ends = !continues_after(tokens, src, it_index);

            if it.kind == .DIRECTIVE && begin == it_index && is_scope_directive(text) ends = true;
        }

        if ends {
            array_add(*out, statement_ending_at(tokens, begin, it_index));
            begin = -1;
        }
    }

    if begin >= 0 array_add(*out, statement_ending_at(tokens, begin, last_code));

    return out;
}

// The statement from token `begin` to token `end`, with the comments that follow it on its line.
statement_ending_at :: (tokens: [] Tok, begin: int, end: int) -> Statement_Range {
    statement: Statement_Range;
    statement.start = tokens[begin].start;
    statement.end = tokens[end].end;
    statement.first_token = begin;

    next := end + 1;

    while next < tokens.count && (tokens[next].kind == .LINE_COMMENT || tokens[next].kind == .BLOCK_COMMENT) && tokens[next].newlines == 0 {
        statement.end = tokens[next].end;
        next += 1;
    }

    return statement;
}

// Whether the statement goes on after the `}` at `close`: `};`, `} else`, or a note after a body.
continues_after :: (tokens: [] Tok, src: string, close: int) -> bool {
    next := close + 1;

    while next < tokens.count && (tokens[next].kind == .LINE_COMMENT || tokens[next].kind == .BLOCK_COMMENT) next += 1;

    if next >= tokens.count return false;

    token := tokens[next];
    text := slice(src, token.start, token.end - token.start);

    if token.kind == .NOTE return true;

    if token.kind == .PUNCT return text == ";";

    return token.kind == .IDENT && text == "else";
}

is_scope_directive :: (text: string) -> bool {
    return text == "#scope_file" || text == "#scope_export" || text == "#scope_module";
}

// Whether the token at `before` lies after a `// jaifmt: off` that no `// jaifmt: on` has ended.
in_off_region :: (tokens: [] Tok, src: string, before: int) -> bool {
    off := false;

    for 0..before - 1 {
        token := tokens[it];

        if token.kind != .LINE_COMMENT continue;

        text := trim(slice(src, token.start + 2, token.end - token.start - 2));

        if !begins_with(text, "jaifmt:") continue;

        word := trim(slice(text, 7, text.count - 7));

        if word == "off" off = true;

        if word == "on" off = false;
    }

    return off;
}

line_begin :: (src: string, at: s64) -> s64 {
    while at > 0 && src[at - 1] != #char "\n" at -= 1;

    return at;
}

line_end :: (src: string, at: s64) -> s64 {
    while at < src.count && src[at] != #char "\n" at += 1;

    return at;
}
