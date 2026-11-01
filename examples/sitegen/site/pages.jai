// Pages: the Markdown files of the source folder.
//
// A page may begin with a front matter block of `key: value` lines between two lines that hold only `---`:
//
//     (a line of three dashes)
//     title: Getting started
//     draft: true
//     (a line of three dashes)
//
// `title` overrides the page's first `#` heading, and `draft: true` leaves the page out of the
// site. Other keys are ignored.

Page :: struct {
    // `guide/intro.md`
    source: string;
    // `guide/intro.html`
    output: string;
    title: string;
    // The rendered Markdown.
    body: string;
}

Front_Matter :: struct {
    title: string;
    draft: bool;
    // The page without its front matter.
    body: string;
}

// Takes a front matter block off the start of `source`. A page without one is all body.
split_front_matter :: (source: string) -> Front_Matter {
    matter: Front_Matter;
    matter.body = source;

    if !begins_with(source, "---\n") && !begins_with(source, "---\r\n") return matter;

    lines := split(source, "\n",, temp);
    end := -1;

    for 1..lines.count - 1 {
        if trim(lines[it]) == "---" {
            end = it;
            break;
        }
    }

    // An opening line with no closing one is not front matter; it is a horizontal rule.
    if end < 0 return matter;

    for 1..end - 1 {
        _, key, value := split_from_left(lines[it], #char ":");
        value = trim(value);

        if trim(key) == "title" matter.title = value;
        else if trim(key) == "draft" matter.draft = value == "true";
    }

    body: String_Builder;

    for end + 1..lines.count - 1 {
        append(*body, lines[it]);

        if it < lines.count - 1 append(*body, "\n");
    }

    matter.body = builder_to_string(*body,, temp);

    return matter;
}

// "guide/intro.md" becomes "guide/intro.html".
page_output :: (source: string) -> string {
    return tprint("%.html", path_strip_extension(source));
} @TempAllocates

// The title a page falls back to: its file name without the extension.
title_from_name :: (source: string) -> string {
    return path_strip_extension(path_filename(source));
}

// The folder a page is in, with a trailing slash: "guide/" for "guide/intro.md", "" at the top.
page_folder :: (source: string) -> string {
    return path_strip_filename(source);
}

// The `<ul>` of links shown in {{ nav }}. `pages` must already be in display order; the link to
// `current` is marked.
render_nav :: (pages: [] *Page, current: *Page) -> string {
    nav: String_Builder;
    root := root_prefix(current.output);

    append(*nav, "<ul>\n");

    for pages {
        append(*nav, "<li><a href=\"");
        append_escaped(*nav, join(root, it.output,, temp));
        append(*nav, "\"");

        if it == current append(*nav, " aria-current=\"page\"");

        append(*nav, ">");
        append_escaped(*nav, it.title);
        append(*nav, "</a></li>\n");
    }

    append(*nav, "</ul>");

    return builder_to_string(*nav,, temp);
}

// Display order: `index.md` first, then by path.
page_before :: (a: *Page, b: *Page) -> s64 {
    a_is_index := a.source == "index.md";
    b_is_index := b.source == "index.md";

    if a_is_index != b_is_index return ifx a_is_index then -1 else 1;

    return compare(a.source, b.source);
}
