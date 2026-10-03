// Reassigned emission helpers; shared records and context remain canonical.
prepend_comment :: (comment: *Comment, text: string) {
    ge_prepend_comment(comment, text);
}

append_prefix_comment :: (builder: *String_Builder, comment: Comment, indent: bool) {
    ge_prefix_comment(builder, comment, indent);
}

append_postfix_comment :: (builder: *String_Builder, comment: Comment) {
    ge_postfix_comment(builder, comment);
}

get_name_alignment :: (declarations: [] *Declaration) -> int {
    width := 0;
    for decl: declarations width = max(width, ge_name_width(decl, false));
    return width;
}

get_name_alignment :: (fields: [] $T, get_declaration: (T) -> *Declaration, ignore_flags := false) -> int {
    assert(get_declaration != null, "Bindings_Generator: missing alignment declaration callback");
    width := 0;
    for field: fields width = max(width, ge_name_width(get_declaration(field), ignore_flags));
    return width;
}

append_indentation :: (builder: *String_Builder) {
    ge_indentation(builder);
}

get_name_for_cursor :: (cur: CXCursor) -> string {
    return ge_cursor_name(cur);
}

maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic_type: Mimic_Spacing_Flags) {
    options := gp_current_options();
    assert(options != null, "Bindings_Generator: spacing requires active options");
    ge_spacing(builder, last_location, location, decl_flags, (options.mimic_spacing_flags & mimic_type) != 0);
}

maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic := true) {
    ge_spacing(builder, last_location, location, decl_flags, mimic);
}

fill_comment :: (comment: *Comment, cur: CXCursor, location: Location) {
    ge_fill_comment(comment, cur, location);
}

fill_location_and_comment_if_needed :: (location: *Location, comment: *Comment, cur: CXCursor) {
    assert(location != null && comment != null, "Bindings_Generator: null source metadata destination");

    if !location.file || clang_equalCursors(cur, clang_getCursorDefinition(cur)) != 0 {
        fill_location(location, cur);
        ge_fill_comment(comment, cur, location.*);
    }
}

debug_print_cursor :: (cur: CXCursor) {
    ge_debug_cursor(cur);
}

#scope_file
#import "Basic";
#load "../internal/developer-tools/generator-emission.jai";
