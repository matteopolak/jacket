// The page template: HTML with `{{ placeholders }}`.
//
//     {{ title }}          the page's title, escaped for HTML
//     {{ content }}        the rendered page
//     {{ nav }}            a list of links to every page, the current one marked
//     {{ root }}           the way back to the site's root: "" or "../" or "../../"
//     {{ path }}           the page's own path, "guide/intro.html"
//     {{ asset a/b.css }}  the published URL of an asset, relative to the page
//     {{ inline a/b.png }} the asset as a data: URI, or its URL when it is too big to inline
//
// A template is parsed once into parts, so a mistake is reported before anything is written.

Part_Kind :: enum {
    TEXT;
    VARIABLE;
    ASSET;
    INLINE;
}

Part :: struct {
    kind: Part_Kind;
    // The literal text, the variable's name, or the asset's source path.
    text: string;
}

Template :: struct {
    parts: [..] Part;
}

// What a page offers to the template.
Page_Values :: struct {
    title: string;
    content: string;
    nav: string;
    path: string;
}

TEMPLATE_VARIABLES :: string.["title", "content", "nav", "root", "path"];

// The template used when the site has no `_template.html`.
DEFAULT_TEMPLATE :: #string END
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{{ title }}</title>
</head>
<body>
<nav>{{ nav }}</nav>
<main>
{{ content }}
</main>
</body>
</html>
END

// Parses `source`. On a mistake the second result says what and on which line, and the template
// is empty. All strings are allocated with the context allocator.
parse_template :: (source: string) -> Template, error: string {
    template: Template;
    rest := source;
    line := 1;

    while rest.count > 0 {
        open := find_index_from_left(rest, "{{");

        if open < 0 {
            array_add(*template.parts, .{ .TEXT, copy_string(rest) });
            break;
        }

        if open > 0 array_add(*template.parts, .{ .TEXT, copy_string(slice(rest, 0, open)) });

        line += count_lines(slice(rest, 0, open));

        close := find_index_from_left(rest, "}}", open);

        if close < 0 {
            free_template(*template);

            return .{}, sprint("line %: `{{` is never closed", line);
        }

        part, error := parse_placeholder(trim(slice(rest, open + 2, close - open - 2)));

        if error.count > 0 {
            free_template(*template);

            message := sprint("line %: %", line, error);
            free(error);

            return .{}, message;
        }

        array_add(*template.parts, part);
        line += count_lines(slice(rest, open, close - open));
        rest = slice(rest, close + 2, rest.count - close - 2);
    }

    return template, "";
}

// Fills the template in for one page. Assets are looked up from the site's root; a missing one
// leaves the placeholder empty and adds a warning to `warnings`. The result is in temporary storage.
expand_template :: (template: *Template, values: Page_Values, assets: *Assets, inline_limit: s64, warnings: *[..] string) -> string {
    root := root_prefix(values.path);
    out: String_Builder;

    for template.parts {
        if it.kind == {
            case .TEXT;
                append(*out, it.text);

            case .VARIABLE;
                if it.text == {
                    case "title"; append_escaped(*out, values.title);
                    case "content"; append(*out, values.content);
                    case "nav"; append(*out, values.nav);
                    case "root"; append(*out, root);
                    case "path"; append(*out, values.path);
                }

            case .ASSET;
                asset := find_asset(assets, it.text);

                if asset append(*out, join(root, asset.output,, temp));
                else array_add(warnings, sprint("the template refers to a missing asset `%`", it.text));

            case .INLINE;
                asset := find_asset(assets, it.text);

                if !asset {
                    array_add(warnings, sprint("the template refers to a missing asset `%`", it.text));
                } else if is_inlinable(asset, inline_limit) {
                    append(*out, data_uri(asset));
                } else {
                    append(*out, join(root, asset.output,, temp));
                }
        }
    }

    return builder_to_string(*out,, temp);
}

free_template :: (template: *Template) {
    for template.parts free(it.text);

    array_free(template.parts);
    template.* = .{};
}

#scope_file

// The contents of one `{{ ... }}`, already trimmed.
parse_placeholder :: (body: string) -> Part, error: string {
    _, word, argument := split_from_left(body, #char " ");
    argument = trim(argument);

    if word == "asset" || word == "inline" {
        if argument.count == 0 return .{}, sprint("`%` needs a file name", word);

        return .{ ifx word == "asset" then Part_Kind.ASSET else .INLINE, copy_string(argument) }, "";
    }

    if argument.count > 0 return .{}, sprint("`%` takes no argument", word);

    for TEMPLATE_VARIABLES {
        if it == word return .{ .VARIABLE, copy_string(word) }, "";
    }

    return .{}, sprint("unknown placeholder `%`", body);
}

count_lines :: (text: string) -> s64 {
    lines := 0;

    for text {
        if it == #char "\n" lines += 1;
    }

    return lines;
}
