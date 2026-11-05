// Documentation declaration layout reuses the normal lex/plan/spacing/emission engine.
// These additional break positions are whitespace-insensitive in the owned parser; suffix
// directives, notes and a continuation after `}` retain their source boundary.
#scope_module

declaration_break_change_allowed :: (src: string, tokens: [] Tok, previous: int, current: int) -> bool {
    if tokens[current].kind == .DIRECTIVE || tokens[current].kind == .NOTE return false;
    if previous < 0 return false;

    before := tokens[previous];
    text := token_text(src, before);

    if before.kind == .DIRECTIVE && trim(slice(text, 1, text.count - 1)) == "elsewhere" return false;
    if before.kind == .PUNCT && text == "}" return false;

    return true;
}

declaration_source_break :: (f: *Formatter, i: int) -> s32 {
    original := f.tokens[i].newlines;
    if original <= 0 return 0;
    if f.verbatim[i] return original;
    if is_comment(f, i) || (i > 0 && is_comment(f, i - 1)) return min(original, 2);

    previous := previous_code(f, i);
    if !break_change_allowed(f.src, f.tokens, previous, i, true)
        && !declaration_break_change_allowed(f.src, f.tokens, previous, i) return 1;

    return 0;
}

// Width wrapping only starts between existing tokens, never inside literals or identifier
// spellings, and never in an adjacency-sensitive directive's flags.
declaration_wrap_before :: (f: *Formatter, i: int) -> bool {
    if i <= 0 || f.tokens[i].kind == .EOF || is_comment(f, i) return false;
    if f.verbatim[i] || f.verbatim[i - 1] || is_comment(f, i - 1) return false;
    if !declaration_break_change_allowed(f.src, f.tokens, i - 1, i) return false;
    if adjacency_matters(f.zone, f.tokens, f.src, i - 1, i) return false;
    if f.tokens[i - 1].kind != .PUNCT return false;
    if is_punct(f, i, ")") || is_punct(f, i, "]") || is_punct(f, i, "}") return false;

    return one_of(text_of(f, i - 1), string.["(", "[", ",", ":", "->"]);
}

// A conservative width until the next legal wrap point. At most one space per boundary is
// counted, so a break may happen slightly early without an input-layout dependency.
declaration_chunk_width :: (f: *Formatter, first: int) -> int {
    width := 0;

    for i: first..f.tokens.count - 1 {
        if f.tokens[i].kind == .EOF || is_comment(f, i) break;
        if i > first && (f.breaks[i] > 0 || declaration_wrap_before(f, i)) break;

        text := text_of(f, i);
        for text {
            if it == #char "\n" return f.config.max_width + 1;
            if (it & 0xC0) != 0x80 width += 1;
        }

        width += 1;
    }

    return max(0, width - 1);
}

// The generic lexer excludes line-comment tail trivia. Declaration docs retain those bytes:
// two spaces are a Markdown hard break, and fenced/indented code may contain trailing tabs.
declaration_emitted_text :: (f: *Formatter, i: int) -> string {
    tok := f.tokens[i];
    if tok.kind != .LINE_COMMENT return text_of(f, i);

    end := tok.end;
    while end < f.src.count && f.src[end] != #char "\n" && f.src[end] != #char "\r" end += 1;

    return slice(f.src, tok.start, end - tok.start);
}

// The end before CR/LF includes significant Markdown tail spaces omitted by Tok.end.
declaration_comment_end :: (f: *Formatter, i: int) -> s64 {
    end := f.tokens[i].end;
    while end < f.src.count && f.src[end] != #char "\n" && f.src[end] != #char "\r" end += 1;
    return end;
}

declaration_comment_body :: (f: *Formatter, i: int) -> string {
    start := f.tokens[i].start + 2;
    end := declaration_comment_end(f, i);
    if start < end && f.src[start] == #char " " start += 1;
    return slice(f.src, start, end - start);
}

declaration_leading_comment :: (f: *Formatter, i: int) -> bool {
    return f.tokens[i].kind == .LINE_COMMENT && (i == 0 || f.tokens[i].newlines > 0)
        && !f.verbatim[i] && !comment_marker(f, i, "off") && !comment_marker(f, i, "on");
}

// Structural Markdown and hard breaks stay byte-for-byte. Their width is a presentation
// concern; reflowing them as prose could change lists, tables, raw HTML or code blocks.
declaration_protected_comment :: (body: string) -> bool {
    if !body.count || !trim(body).count return true;
    if body[0] == #char "\t" || begins_with(body, "    ") return true;
    if ends_with(body, "  ") || ends_with(body, "\\") return true;

    text := trim(body);
    if contains(text, "|") || contains(text, "<") return true;
    if contains("#>*-+[=/", slice(text, 0, 1)) return true;

    // One/two backticks are inline code even at a wrapped line's beginning. Only a
    // fence-length run introduces a block; treating every backtick as structural made
    // the conservation key depend on where prose happened to wrap.
    if text[0] == #char "`" || text[0] == #char "~" {
        run := 0;
        while run < text.count && text[run] == text[0] run += 1;
        if run >= 3 return true;
    }

    at := 0;
    while at < text.count && text[at] >= #char "0" && text[at] <= #char "9" at += 1;
    if at > 0 && at + 1 < text.count && (text[at] == #char "." || text[at] == #char ")")
        && (text[at + 1] == #char " " || text[at + 1] == #char "\t") return true;

    return false;
}

// Whitespace separates prose atoms, except inside a balanced inline-code delimiter. An
// unmatched delimiter keeps the original paragraph rather than guessing its Markdown role.
declaration_comment_atoms :: (text: string, atoms: *[..] string) -> bool {
    at: s64 = 0;
    while at < text.count {
        while at < text.count && (text[at] == #char " " || text[at] == #char "\t") at += 1;
        if at >= text.count break;

        start := at;

        while at < text.count && text[at] != #char " " && text[at] != #char "\t" {
            if text[at] != #char "`" {
                at += 1;
                continue;
            }

            run := 0;
            while at + run < text.count && text[at + run] == #char "`" run += 1;

            scan := at + run;
            closed := false;

            while scan < text.count {
                if text[scan] != #char "`" {
                    scan += 1;
                    continue;
                }

                count := 0;
                while scan + count < text.count && text[scan + count] == #char "`" count += 1;

                scan += count;

                if count == run {
                    closed = true;
                    break;
                }
            }

            if !closed return false;

            at = scan;
        }

        array_add(atoms, slice(text, start, at - start));
    }

    return true;
}

// Reflow only leading prose paragraphs. Code bytes, trailing comments, paragraph boundaries
// and protected Markdown are copied. Width zero gives a canonical single-line paragraph used
// by the independent whole-text conservation check after bounded wrapping.
declaration_reflow_comments :: (text: string, max_width: int) -> string {
    parsed := lex(text);

    defer array_free(parsed.tokens);
    f: Formatter;
    f.src = text;
    f.tokens = parsed.tokens;
    mark_verbatim(*f);
    defer array_free(f.verbatim);
    out: String_Builder;

    defer free_buffers(*out);
    newline := ifx uses_crlf(text) then "\r\n" else "\n";
    cursor: s64 = 0;
    i := 0;

    while i < f.tokens.count {
        if !declaration_leading_comment(*f, i) {
            i += 1;
            continue;
        }

        last := i;
        protected := false;
        fence: u8;
        fence_count: s64 = 0;

        while true {
            body := declaration_comment_body(*f, last);
            blank := !trim(body).count;

            if last > i && blank && fence == 0 {
                last -= 1;
                break;
            }

            if declaration_protected_comment(body) protected = true;

            trimmed := trim(body);
            if trimmed.count && (trimmed[0] == #char "`" || trimmed[0] == #char "~") {
                count: s64 = 0;
                while count < trimmed.count && trimmed[count] == trimmed[0] count += 1;

                if count >= 3 {
                    if fence == 0 {
                        fence = trimmed[0];
                        fence_count = count;
                    } else if fence == trimmed[0] && count >= fence_count && !trim(slice(trimmed, count, trimmed.count - count)).count {
                        fence = 0;
                    }
                }
            }

            if blank && fence == 0 break;

            next := last + 1;
            if next >= f.tokens.count || !declaration_leading_comment(*f, next) || f.tokens[next].newlines != 1 break;

            last = next;
        }

        end := declaration_comment_end(*f, last);

        append(*out, slice(text, cursor, f.tokens[i].start - cursor));
        paragraph: String_Builder;

        for line: i..last {
            if line > i append(*paragraph, " ");
            append(*paragraph, declaration_comment_body(*f, line));
        }

        joined := builder_to_string(*paragraph);

        free_buffers(*paragraph);
        atoms: [..] string;
        if !protected && !declaration_comment_atoms(joined, *atoms) protected = true;

        if protected {
            append(*out, slice(text, f.tokens[i].start, end - f.tokens[i].start));
        } else {
            indent := f.tokens[i].column;
            column: int = indent + 3;

            append(*out, "//");
            if atoms.count append(*out, " ");

            for word, index: atoms {
                width := 0;
                for word if (it & 0xC0) != 0x80 width += 1;

                if index > 0 && max_width > 0 && column + 1 + width > max_width {
                    append(*out, newline);
                    for 1..indent append(*out, " ");
                    append(*out, "// ");
                    column = indent + 3;
                } else if index > 0 {
                    append(*out, " ");
                    column += 1;
                }

                append(*out, word);
                column += width;
            }
        }

        array_free(atoms);
        free(joined);
        cursor = end;
        i = last + 1;
    }

    append(*out, slice(text, cursor, text.count - cursor));

    return builder_to_string(*out);
}
