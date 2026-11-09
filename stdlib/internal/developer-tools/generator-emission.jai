#scope_module
#import "Basic";
#import "String";
GE_Basic :: #import "Basic";
GE_Format :: #import "Extensions/Jai_Format";
GE_File :: #import "File";
GE_Pool :: #import "Pool";

ge_prepend_comment :: (comment: *Comment, text: string) {
    assert(comment != null, "Bindings_Generator: null comment destination");
    if !text.count return;

    if !comment.text.count {
        comment.text = text;
        return;
    }

    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, text);

    if comment.text.count {
        append(*builder, " ");
        append(*builder, comment.text);
    }

    comment.text = builder_to_string(*builder);
}

ge_is_prefix :: (comment: Comment) -> bool {
    text := trim_left(comment.text);
    if begins_with(text, "///<") || begins_with(text, "//!<") return false;
    return comment.location.line != 0 || contains(comment.text, "\n");
}

ge_indentation :: (builder: *String_Builder) {
    generator := gp_current_generator();
    assert(generator != null, "Bindings_Generator: indentation requires an active generator");
    assert(generator.tab_depth >= 0, "Bindings_Generator: negative indentation depth");
    for 0..generator.tab_depth - 1 append(builder, TAB_STRING);
}

// Strip only the marker's padding; indentation after it belongs to Markdown/code.
ge_comment_body :: (line: string) -> string {
    line := trim_right(line, " \t\r");
    start := trim_left(line, " \t");

    if start == "*/" return "";

    if begins_with(start, "/*") {
        line = slice(start, 2, start.count - 2);
        if begins_with(line, "*") || begins_with(line, "!") line = slice(line, 1, line.count - 1);
    } else if begins_with(start, "//") {
        line = slice(start, 2, start.count - 2);
        if begins_with(line, "/") || begins_with(line, "!") line = slice(line, 1, line.count - 1);
        if begins_with(line, "<") line = slice(line, 1, line.count - 1);
    } else if begins_with(start, "*") {
        line = slice(start, 1, start.count - 1);
    }

    if ends_with(line, "*/") line = slice(line, 0, line.count - 2);
    if begins_with(line, " ") line = slice(line, 1, line.count - 1);

    return trim_right(line, " \t\r");
}

// Paragraphs wrap at words; list continuations stay inside their Markdown item.
ge_comment_paragraph :: (builder: *String_Builder, paragraph: *String_Builder, width: int) {
    text := builder_to_string(paragraph);

    defer free(text);
    if !text.count return;

    hanging := ifx begins_with(text, "- ") then "  " else "";
    words := split(text, " ");

    defer array_free(words);
    column := 0;

    for word: words {
        if !word.count continue;

        if column && column + 1 + word.count > width {
            append(builder, "\n");
            append(builder, hanging);
            column = hanging.count;
        } else if column {
            append(builder, " ");
            column += 1;
        }

        append(builder, word);
        column += word.count;
    }

    append(builder, "\n");
}

// Separate the next block with one empty line, unless it opens the doc or follows one.
ge_comment_break :: (output: *String_Builder) {
    length := builder_string_length(output);
    if !length return;

    text := builder_to_string(output, do_reset = false);

    defer free(text);
    if !ends_with(text, "\n\n") append(output, "\n");
}

ge_comment_text :: (raw: string, width: int) -> string {
    normalized: String_Builder;

    defer free_buffers(*normalized);
    unix := replace(raw, "\r\n", "\n");

    defer free(unix);
    canonical := replace(unix, "\r", "\n");

    defer free(canonical);
    lines := split(canonical, "\n");

    defer array_free(lines);

    for line: lines {
        append(*normalized, ge_comment_body(line));
        append(*normalized, "\n");
    }

    clean := builder_to_string(*normalized);

    defer free(clean);
    bodies := split(trim(clean, "\r\n"), "\n");

    defer array_free(bodies);
    output, paragraph: String_Builder;

    defer free_buffers(*output);
    defer free_buffers(*paragraph);
    code := false;
    listing := false;

    for body: bodies {
        text := trim(body);
        tagged := begins_with(text, "@") || begins_with(text, "\\");
        tag, rest: string;

        if tagged {
            end := 1;
            while end < text.count && text[end] != #char " " && text[end] != #char "\t" && text[end] != #char "[" end += 1;
            tag = slice(text, 1, end - 1);
            rest = trim_left(slice(text, end, text.count - end));
        }

        if code {
            if tag == "endcode" || begins_with(text, "```") {
                append(*output, "```\n");
                code = false;
            } else {
                append(*output, body);
                append(*output, "\n");
            }

            continue;
        }

        structural := tagged || !text.count || begins_with(text, "```") || begins_with(text, "- ") || begins_with(body, "    ") || begins_with(body, "\t");
        if structural ge_comment_paragraph(*output, *paragraph, width);
        // Each tag is its own Markdown block; consecutive params stay one list.
        if tagged && !(tag == "param" && listing) ge_comment_break(*output);
        if structural listing = tag == "param";

        if !text.count {
            append(*output, "\n");
            continue;
        }

        if tag == "code" || begins_with(text, "```") {
            append(*output, ifx tagged then "```c\n" else tprint("%\n", body));
            code = true;
            continue;
        }

        if !tagged && (begins_with(body, "    ") || begins_with(body, "\t")) {
            append(*output, body);
            append(*output, "\n");
            continue;
        }

        if tagged {
            if tag == "brief" text = rest;
            else if tag == "param" {
                if begins_with(rest, "[") {
                    end := 0;
                    while end < rest.count && rest[end] != #char "]" end += 1;
                    if end < rest.count rest = trim_left(slice(rest, end + 1, rest.count - end - 1));
                }

                end := 0;
                while end < rest.count && rest[end] != #char " " && rest[end] != #char "\t" end += 1;

                text = tprint("- %: %", slice(rest, 0, end), trim_left(slice(rest, end, rest.count - end)));
            } else if tag == "return" || tag == "returns" text = tprint("Returns %", rest);
            else if tag == "see" || tag == "sa" text = tprint("See also: %", rest);
            else if tag == "note" || tag == "warning" || tag == "deprecated" {
                label := ifx tag == "note" then "Note" else ifx tag == "warning" then "Warning" else "Deprecated";
                text = tprint("**%** %", label, rest);
            } else text = slice(text, 1, text.count - 1);
        }

        if builder_string_length(*paragraph) append(*paragraph, " ");

        append(*paragraph, text);
    }

    ge_comment_paragraph(*output, *paragraph, width);
    result := builder_to_string(*output);
    // Ignore delimiter-only edge lines, not meaningful empty lines within the doc.
    trimmed := copy_string(trim(result, "\n\r"));

    free(result);

    return trimmed;
}

ge_comments_enabled :: () -> bool {
    options := gp_current_options();
    return !options || options.try_to_preserve_comments;
}

// Resolve the formatter settings once per generator, from the caller's directory.
ge_comment_width :: () -> int {
    generator := gp_current_generator();
    defaults: GE_Format.Format_Config;

    if !generator return defaults.max_width;
    if generator.comment_max_width return generator.comment_max_width;

    pool: GE_Pool.Pool;

    GE_Pool.set_allocators(*pool, context.allocator);
    defer GE_Pool.release(*pool);
    push_context, defer_pop;
    push_allocator(GE_Pool.pool_allocator_proc, *pool);
    directory := GE_Basic.get_working_directory();
    if !directory.count return 0;

    reader :: (path: string) -> string, bool {
        return GE_File.read_entire_file(path);
    }

    config, _, ok, error := GE_Format.format_find_config(directory, reader);

    if !ok {
        GE_Basic.log_error("Bindings_Generator: formatter config: %\n", error);
        return 0;
    }

    generator.comment_max_width = config.max_width;

    return config.max_width;
}

ge_prefix_comment :: (builder: *String_Builder, comment: Comment, indent: bool) {
    if !ge_comments_enabled() || !ge_is_prefix(comment) || !comment.text.count return;

    width := ge_comment_width();
    if !width {
        builder.failed = true;
        return;
    }

    generator := gp_current_generator();
    depth := ifx generator then generator.tab_depth else 0;
    text := ge_comment_text(comment.text, max(1, width - depth * TAB_STRING.count - 3));

    defer free(text);
    lines := split(text, "\n");

    defer array_free(lines);

    for line: lines {
        append(builder, "//");

        if line.count {
            append(builder, " ");
            append(builder, line);
        }

        append(builder, "\n");
        if indent ge_indentation(builder);
    }
}

ge_postfix_comment :: (builder: *String_Builder, comment: Comment) {
    if !ge_comments_enabled() || !comment.text.count || ge_is_prefix(comment) return;

    width := ge_comment_width();
    if !width {
        builder.failed = true;
        return;
    }

    // The doc reader takes a field's trailing doc from this line only. Moving a
    // continuation below it would document the next field instead.
    text := ge_comment_text(comment.text, width - 3);

    defer free(text);
    append(builder, " // ");
    for byte: text append(builder, ifx byte == #char "\n" then #char " " else byte);
}

ge_name_width :: (decl: *Declaration, ignore_flags: bool) -> int {
    assert(decl != null, "Bindings_Generator: null declaration in alignment input");
    if decl.kind != .DECLARATION && decl.kind != .TYPEDEF && decl.kind != .MACRO_DEFINITION && decl.kind != .BITFIELD return 0;
    // These eligible non-function categories move only for extern classification.
    // Functions are excluded above regardless of the move-functions option.
    if decl.storage_class == .Extern return 0;

    width := decl.output_name.count;

    if !ignore_flags {
        if (decl.decl_flags & .QUALIFIER_AS) != 0 width += "#as ".count;

        if (decl.decl_flags & .QUALIFIER_USING) != 0 {
            width += "using ".count;
            if decl.using_except.count width += ",except()".count + decl.using_except.count;
        }
    }

    return width;
}

ge_owned_clang_string :: (native: CXString) -> string {
    defer clang_disposeString(native);
    pointer := clang_getCString(native);
    if !pointer return "";
    return copy_string(to_string(pointer));
}

ge_cursor_name :: (cur: CXCursor) -> string {
    name := ge_owned_clang_string(clang_getCursorSpelling(cur));
    if contains(name, " at ") {
        free(name);
        return "";
    }

    return name;
}

ge_spacing :: (builder: *String_Builder, last: *Location, location: Location, flags: Declaration_Flags, mimic: bool) {
    assert(last != null, "Bindings_Generator: null previous spacing location");
    previous := last.*;

    last.* = location;
    if (flags & .NO_SPACING) == 0 append(builder, "\n");

    // Subtraction only after ordering prevents unsigned wrap at u32's boundary.
    separated := location.line != previous.line && (location.line < previous.line || location.line - previous.line != 1);
    if mimic && previous.file && (previous.file != location.file || separated) append(builder, "\n");
}

ge_fill_comment :: (comment: *Comment, cur: CXCursor, location: Location) {
    assert(comment != null, "Bindings_Generator: null comment destination");
    native := clang_Cursor_getRawCommentText(cur);

    defer clang_disposeString(native);
    pointer := clang_getCString(native);
    if !pointer return;

    text := trim(to_string(pointer));
    if !text.count return;

    comment.text = copy_string(text);
    start := clang_getRangeStart(clang_Cursor_getCommentRange(cur));
    beginning: Location;

    clang_getExpansionLocation(start, *beginning.file, *beginning.line, *beginning.column, null);
    if beginning.line < location.line comment.location = beginning;
}

GE_Debug_Walk :: struct {
    saved_context: #Context;
    depth: int;
}

ge_debug_cursor :: (cur: CXCursor) {
    walk: GE_Debug_Walk;
    walk.saved_context = context;
    clang_visitChildren(cur, ge_debug_child, *walk);
}

ge_debug_child :: (cur: CXCursor, _parent: CXCursor, data: *void) -> CXChildVisitResult #c_call {
    walk := cast(*GE_Debug_Walk) data;

    push_context {
        context = walk.saved_context;
        kind := ge_owned_clang_string(clang_getCursorKindSpelling(cur.kind));

        defer free(kind);
        name := ge_owned_clang_string(clang_getCursorSpelling(cur));

        defer free(name);
        builder: String_Builder;

        defer free_buffers(*builder);
        for 0..walk.depth - 1 append(*builder, TAB_STRING);

        append(*builder, kind);
        append(*builder, " ");
        append(*builder, name);
        append(*builder, "\n");
        message := builder_to_string(*builder);

        defer free(message);
        GE_Basic.log("%", message);
        walk.depth += 1;
        defer walk.depth -= 1;
        clang_visitChildren(cur, ge_debug_child, walk);
    }

    return .Continue;
}
