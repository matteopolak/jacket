// The structure of a document: which lines are a heading, a code fence, a quote, a list or part of
// a paragraph. Each block is rendered as soon as it ends; nothing is kept as a tree.

Rendered :: struct {
    // The document as HTML. The caller frees it.
    html: string;
    // The text of the first `#` heading, or "" when there is none. The caller frees it.
    title: string;
}

// Turns `source` into HTML. `options` may be null.
render_markdown :: (source: string, options: *Options = null) -> Rendered {
    no_options: Options;
    state := Block_State.{ options = ifx options then options else *no_options };
    out: String_Builder;

    lines := split(source, "\n",, temp);

    for *lines {
        if ends_with(it.*, "\r") it.* = slice(it.*, 0, it.count - 1);
    }

    render_blocks(*out, lines, false, *state);

    return .{ html = builder_to_string(*out), title = copy_string(state.title) };
}

#scope_file

Block_State :: struct {
    options: *Options;
    title: string;
}

// What a line says about the list it might start or continue.
Item_Marker :: struct {
    // Leading spaces.
    indent: s64;
    ordered: bool;
    // Where the item's text starts on the line.
    text_at: s64;
}

render_blocks :: (out: *String_Builder, lines: [] string, tight: bool, state: *Block_State) {
    paragraph: [..] string;
    paragraph.allocator = temp;

    i := 0;

    while i < lines.count {
        line := lines[i];
        text := trim(line);
        _, is_item := item_marker(line);

        if text.count == 0 {
            flush_paragraph(out, *paragraph, tight, state);
            i += 1;
        } else if begins_with(text, "```") {
            flush_paragraph(out, *paragraph, tight, state);
            i = render_fence(out, lines, i);
        } else if heading_level(text) > 0 {
            flush_paragraph(out, *paragraph, tight, state);
            render_heading(out, text, state);
            i += 1;
        } else if is_rule(text) {
            flush_paragraph(out, *paragraph, tight, state);
            append(out, "<hr>\n");
            i += 1;
        } else if text[0] == #char ">" {
            flush_paragraph(out, *paragraph, tight, state);
            i = render_quote(out, lines, i, state);
        } else if is_item {
            flush_paragraph(out, *paragraph, tight, state);
            i = render_list(out, lines, i, state);
        } else {
            array_add(*paragraph, text);
            i += 1;
        }
    }

    flush_paragraph(out, *paragraph, tight, state);
}

flush_paragraph :: (out: *String_Builder, paragraph: *[..] string, tight: bool, state: *Block_State) {
    if paragraph.count == 0 return;

    if !tight append(out, "<p>");

    render_inline(out, join(..paragraph.*, separator = "\n",, temp), state.options);

    if !tight append(out, "</p>");

    append(out, "\n");

    paragraph.count = 0;
}

// The number of `#` at the start of `text` when they are followed by a space, else 0.
heading_level :: (text: string) -> s64 {
    level := 0;

    while level < text.count && text[level] == #char "#" level += 1;

    if level == 0 || level > 6 return 0;
    if level < text.count && text[level] != #char " " return 0;

    return level;
}

render_heading :: (out: *String_Builder, text: string, state: *Block_State) {
    level := heading_level(text);
    content := trim_right(trim(slice(text, level, text.count - level)), "#");
    content = trim(content);

    if level == 1 && state.title.count == 0 state.title = copy_string(content,, temp);

    print_to_builder(out, "<h% id=\"%\">", level, slugify(content));
    render_inline(out, content, state.options);
    print_to_builder(out, "</h%>\n", level);
}

// Three or more of the same `-`, `*` or `_` and nothing else.
is_rule :: (text: string) -> bool {
    if text.count < 3 return false;
    if text[0] != #char "-" && text[0] != #char "*" && text[0] != #char "_" return false;

    for 1..text.count - 1 {
        if text[it] != text[0] return false;
    }

    return true;
}

// Renders the fence that starts at `lines[start]` and returns the index after its closing line.
// A fence that is never closed runs to the end of the document.
render_fence :: (out: *String_Builder, lines: [] string, start: s64) -> s64 {
    language := trim(slice(trim(lines[start]), 3, trim(lines[start]).count - 3));

    if language.count > 0 {
        print_to_builder(out, "<pre><code class=\"language-%\">", language);
    } else {
        append(out, "<pre><code>");
    }

    i := start + 1;

    while i < lines.count && !begins_with(trim(lines[i]), "```") {
        append_escaped(out, lines[i]);
        append(out, "\n");
        i += 1;
    }

    append(out, "</code></pre>\n");

    return min(i + 1, lines.count);
}

// Renders the run of `>` lines from `start` as a block quote with its own blocks inside.
render_quote :: (out: *String_Builder, lines: [] string, start: s64, state: *Block_State) -> s64 {
    inner: [..] string;
    inner.allocator = temp;

    i := start;

    while i < lines.count {
        text := trim(lines[i]);

        if text.count == 0 || text[0] != #char ">" break;

        text = slice(text, 1, text.count - 1);

        if begins_with(text, " ") text = slice(text, 1, text.count - 1);

        array_add(*inner, text);
        i += 1;
    }

    append(out, "<blockquote>\n");
    render_blocks(out, inner, false, state);
    append(out, "</blockquote>\n");

    return i;
}

// Whether `line` starts a list item, and what its marker looks like.
item_marker :: (line: string) -> Item_Marker, bool {
    marker: Item_Marker;

    while marker.indent < line.count && line[marker.indent] == #char " " marker.indent += 1;

    at := marker.indent;

    if at + 1 < line.count && (line[at] == #char "-" || line[at] == #char "*" || line[at] == #char "+") && line[at + 1] == #char " " {
        marker.text_at = at + 2;

        return marker, true;
    }

    digits := 0;

    while at + digits < line.count && line[at + digits] >= #char "0" && line[at + digits] <= #char "9" digits += 1;

    if digits > 0 && at + digits + 1 < line.count && line[at + digits] == #char "." && line[at + digits + 1] == #char " " {
        marker.ordered = true;
        marker.text_at = at + digits + 2;

        return marker, true;
    }

    return marker, false;
}

// Whether `line` is the kind of line that ends a list rather than continuing an item.
starts_other_block :: (line: string) -> bool {
    text := trim(line);

    return begins_with(text, "```") || begins_with(text, ">") || heading_level(text) > 0 || is_rule(text);
}

// Renders the list that starts at `lines[start]` and returns the index after it. Lines indented
// under an item belong to it, so a nested list is just that item's own blocks; a blank line ends
// the list unless another item follows.
render_list :: (out: *String_Builder, lines: [] string, start: s64, state: *Block_State) -> s64 {
    first, _ := item_marker(lines[start]);

    tag := ifx first.ordered then "ol" else "ul";

    print_to_builder(out, "<%>\n", tag);

    item: [..] string;
    item.allocator = temp;

    i := start;

    while i < lines.count {
        line := lines[i];
        marker, is_item := item_marker(line);

        if line.count == 0 || trim(line).count == 0 {
            // Look past blank lines for another item of this list.
            next := i;

            while next < lines.count && trim(lines[next]).count == 0 next += 1;

            if next >= lines.count break;

            next_marker, next_is_item := item_marker(lines[next]);

            if !next_is_item || next_marker.indent > first.indent || next_marker.ordered != first.ordered break;

            i = next;
            continue;
        }

        // A bullet after a numbered item, or the other way round, starts another list.
        if is_item && marker.indent <= first.indent && marker.ordered != first.ordered break;

        nested := is_item && marker.indent > first.indent;
        lazy := !is_item && !starts_other_block(line);

        if is_item && marker.indent <= first.indent {
            if item.count > 0 flush_item(out, *item, state);

            array_add(*item, slice(line, marker.text_at, line.count - marker.text_at));
        } else if item.count > 0 && (nested || lazy) {
            array_add(*item, drop_indent(line, first.text_at - first.indent));
        } else {
            break;
        }

        i += 1;
    }

    if item.count > 0 flush_item(out, *item, state);

    print_to_builder(out, "</%>\n", tag);

    return i;
}

flush_item :: (out: *String_Builder, item: *[..] string, state: *Block_State) {
    inner: String_Builder;

    render_blocks(*inner, item.*, true, state);

    append(out, "<li>");
    append(out, trim_right(builder_to_string(*inner,, temp), "\n"));
    append(out, "</li>\n");

    item.count = 0;
}

// `line` without up to `width` leading spaces.
drop_indent :: (line: string, width: s64) -> string {
    at := 0;

    while at < width && at < line.count && line[at] == #char " " at += 1;

    return slice(line, at, line.count - at);
}
