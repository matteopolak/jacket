// The text inside a block: code spans, emphasis, links and images.

Link_Kind :: enum {
    LINK;
    IMAGE;
}

// Hooks that let the caller decide what a document's links and images point at.
Options :: struct {
    // Returns the URL to write for `url`, in temporary storage (or `url` itself). Left null, every
    // URL is written as it was typed.
    rewrite_url: (url: string, kind: Link_Kind, data: *void) -> string;
    data: *void;
}

// Appends the HTML for one line or paragraph of inline Markdown.
render_inline :: (out: *String_Builder, text: string, options: *Options) {
    i := 0;

    while i < text.count {
        c := text[i];
        rest := slice(text, i, text.count - i);

        if c == #char "\\" && i + 1 < text.count && is_ascii_punctuation(text[i + 1]) {
            append_escaped(out, slice(text, i + 1, 1));
            i += 2;
        } else if c == #char "`" && inline_code(out, rest, *i) {} else if c == #char "!" && begins_with(rest, "![") && inline_link(out, rest, .IMAGE, options, *i) {} else if c == #char "[" && inline_link(out, rest, .LINK, options, *i) {} else if (c == #char "*" || c == #char "_") && inline_emphasis(out, text, i, options, *i) {} else {
            append_escaped(out, slice(text, i, 1));
            i += 1;
        }
    }
}

#scope_file

// A `code span`. Advances `at` past it and returns true, or returns false when the backtick is not closed.
inline_code :: (out: *String_Builder, rest: string, at: *s64) -> bool {
    end := find_index_from_left(rest, #char "`", 1);

    if end < 0 return false;

    append(out, "<code>");
    append_escaped(out, slice(rest, 1, end - 1));
    append(out, "</code>");
    at.* += end + 1;

    return true;
}

// `[text](url)` or `![alt](url)` at the start of `rest`.
inline_link :: (out: *String_Builder, rest: string, kind: Link_Kind, options: *Options, at: *s64) -> bool {
    open := ifx kind == .IMAGE then 1 else 0;
    close := find_index_from_left(rest, #char "]", open + 1);

    if close < 0 || close + 1 >= rest.count || rest[close + 1] != #char "(" return false;

    end := find_index_from_left(rest, #char ")", close + 2);

    if end < 0 return false;

    label := slice(rest, open + 1, close - open - 1);
    url := trim(slice(rest, close + 2, end - close - 2));

    if options.rewrite_url url = options.rewrite_url(url, kind, options.data);

    if kind == .IMAGE {
        append(out, "<img src=\"");
        append_escaped(out, url);
        append(out, "\" alt=\"");
        append_escaped(out, label);
        append(out, "\">");
    } else {
        append(out, "<a href=\"");
        append_escaped(out, url);
        append(out, "\">");
        render_inline(out, label, options);
        append(out, "</a>");
    }

    at.* += end + 1;

    return true;
}

// `*em*`, `_em_` or `**strong**` starting at `text[start]`. Sets `at` past it.
inline_emphasis :: (out: *String_Builder, text: string, start: s64, options: *Options, at: *s64) -> bool {
    marker := text[start];
    strong := start + 1 < text.count && text[start + 1] == marker;
    width := ifx strong then 2 else 1;
    inner_start := start + width;

    // `snake_case_name` is not emphasis, and neither is a lone star followed by a space.
    if marker == #char "_" && start > 0 && is_word_character(text[start - 1]) return false;
    if inner_start >= text.count || text[inner_start] == #char " " return false;

    end := closing_marker(text, inner_start, marker, width);

    if end < 0 || end == inner_start return false;
    if marker == #char "_" && end + width < text.count && is_word_character(text[end + width]) return false;

    tag := ifx strong then "strong" else "em";

    print_to_builder(out, "<%>", tag);
    render_inline(out, slice(text, inner_start, end - inner_start), options);
    print_to_builder(out, "</%>", tag);

    at.* = end + width;

    return true;
}

// Where the `marker` that closes an emphasis starting at `from` is, or -1. A closing marker follows
// a non-space; a single one is not the start of a double one; a backslash hides the next byte.
closing_marker :: (text: string, from: s64, marker: u8, width: s64) -> s64 {
    end := from;

    while end < text.count {
        if text[end] == #char "\\" {
            end += 2;
            continue;
        }

        run := 0;

        while end + run < text.count && text[end + run] == marker run += 1;

        if run >= width && text[end - 1] != #char " " && (width == 2 || run == 1) return end;

        end += max(run, 1);
    }

    return -1;
}
