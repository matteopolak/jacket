#scope_module
#import "Basic";
#import "String";
GE_Basic :: #import "Basic";

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
    return comment.location.line != 0 || contains(comment.text, "\n");
}

ge_indentation :: (builder: *String_Builder) {
    generator := gp_current_generator();
    assert(generator != null, "Bindings_Generator: indentation requires an active generator");
    assert(generator.tab_depth >= 0, "Bindings_Generator: negative indentation depth");
    for 0..generator.tab_depth - 1 append(builder, TAB_STRING);
}

ge_prefix_comment :: (builder: *String_Builder, comment: Comment, indent: bool) {
    if !ge_is_prefix(comment) return;

    lines := split(comment.text, "\n");

    defer array_free(lines);

    for line: lines {
        append(builder, trim(line, " \t\r"));
        append(builder, "\n");
        if indent ge_indentation(builder);
    }
}

ge_postfix_comment :: (builder: *String_Builder, comment: Comment) {
    if !comment.text.count || ge_is_prefix(comment) return;

    append(builder, " ");

    for byte, index: comment.text {
        if byte == #char "\r" {
            append(builder, "\n");
        } else if byte != #char "\n" || index == 0 || comment.text[index - 1] != #char "\r" {
            append(builder, byte);
        }
    }
}

ge_name_width :: (decl: *Declaration, ignore_flags: bool) -> int {
    assert(decl != null, "Bindings_Generator: null declaration in alignment input");
    if decl.kind != .DECLARATION && decl.kind != .TYPEDEF && decl.kind != .MACRO_DEFINITION && decl.kind != .BITFIELD return 0;
    // These eligible non-function categories move only for extern classification.
    // Functions are excluded above regardless of the move-functions option.
    if decl.storage_class == .Extern return 0;

    width := decl.output_name.count;

    if !ignore_flags {
        if (decl.decl_flags & .QUALIFIER_USING) != 0 width += 6;
        else if (decl.decl_flags & .QUALIFIER_AS) != 0 width += 4;
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
