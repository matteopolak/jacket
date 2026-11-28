// Puts `text` in front of the text of `comment`, separated by a space.
//
// Does nothing when `text` is empty. Asserts that `comment` is not null.
prepend_comment :: (comment: *Comment, text: string) {
    ge_prepend_comment(comment, text);
}

// Writes `comment` as `//` lines above a declaration, converting its C documentation markup to Markdown.
//
// Doxygen tags such as `@brief`, `@param` and `@return` become Markdown text, and lines wrap at the maximum
// width of the `jaifmt` configuration found from the working directory. With `indent` set, the current
// indentation follows each line. Writes nothing when `try_to_preserve_comments` is off, when the text is
// empty, or when `comment` is a trailing comment (see [append_postfix_comment]). Sets `builder.failed` when
// the formatter configuration cannot be read.
append_prefix_comment :: (builder: *String_Builder, comment: Comment, indent: bool) {
    ge_prefix_comment(builder, comment, indent);
}

// Writes `comment` after a declaration on the same line, as ` // text`.
//
// Line breaks in the text become spaces. Writes nothing when `try_to_preserve_comments` is off, when the text
// is empty, or when `comment` is a header comment: one that has a source line or spans several lines, unless
// it starts with `///<` or `//!<`. Sets `builder.failed` when the formatter configuration cannot be read.
append_postfix_comment :: (builder: *String_Builder, comment: Comment) {
    ge_postfix_comment(builder, comment);
}

// Returns the width of the longest name among `declarations`, for padding names into a column.
//
// Only plain declarations, typedefs, macros and bitfields count, and `extern` declarations are skipped. The
// width includes the `#as ` and `using ` qualifiers and the `except(...)` list of a `using` that will be
// printed.
get_name_alignment :: (declarations: [] *Declaration) -> int {
    width := 0;
    for decl: declarations width = max(width, ge_name_width(decl, false));
    return width;
}

// Returns the width of the longest name among the declarations that `get_declaration` yields for `fields`.
//
// Counts names as the other overload does. With `ignore_flags`, the `#as ` and `using ` qualifiers are left
// out of the width. Asserts that `get_declaration` is not null.
get_name_alignment :: (fields: [] $T, get_declaration: (T) -> *Declaration, ignore_flags := false) -> int {
    assert(get_declaration != null, "Bindings_Generator: missing alignment declaration callback");
    width := 0;
    for field: fields width = max(width, ge_name_width(get_declaration(field), ignore_flags));
    return width;
}

// Appends one [TAB_STRING] per level of the active generator's current nesting depth.
//
// Asserts that a generator is active.
append_indentation :: (builder: *String_Builder) {
    ge_indentation(builder);
}

// Returns a copy of the spelling of `cur`, or an empty string when libclang gives it a placeholder name.
//
// libclang names anonymous declarations with a text containing ` at `; that counts as no name. The caller
// frees the result.
get_name_for_cursor :: (cur: CXCursor) -> string {
    return ge_cursor_name(cur);
}

// Writes the blank lines that separate a declaration at `location` from the previous one at `last_location`,
// and records `location` as the new previous location.
//
// One newline is written unless `decl_flags` has `.NO_SPACING`. A second one is written to keep the gap of
// the original headers: when the declarations are in different files or not on adjacent lines. That second
// newline is used only when the active options' `mimic_spacing_flags` include `mimic_type`. Asserts that
// options are active.
maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic_type: Mimic_Spacing_Flags) {
    options := gp_current_options();
    assert(options != null, "Bindings_Generator: spacing requires active options");
    ge_spacing(builder, last_location, location, decl_flags, (options.mimic_spacing_flags & mimic_type) != 0);
}

// Writes the blank lines that separate a declaration at `location` from the previous one at `last_location`,
// and records `location` as the new previous location.
//
// Behaves like the overload taking a [Mimic_Spacing_Flags] value, with `mimic` saying directly whether the
// gap of the original headers is kept.
maybe_add_spacing :: (builder: *String_Builder, last_location: *Location, location: Location, decl_flags: Declaration_Flags, mimic := true) {
    ge_spacing(builder, last_location, location, decl_flags, mimic);
}

// Reads the documentation comment of `cur` into `comment`.
//
// Leaves `comment` unchanged when the cursor has no comment. The text is a copy. Its `location` is set only
// when the comment starts on a line before `location`.
fill_comment :: (comment: *Comment, cur: CXCursor, location: Location) {
    ge_fill_comment(comment, cur, location);
}

// Sets `location` and `comment` from `cur` when `location` is still unset or `cur` is a definition.
//
// A declaration that is not a definition does not overwrite a location already taken from an earlier one.
// Asserts that neither pointer is null.
fill_location_and_comment_if_needed :: (location: *Location, comment: *Comment, cur: CXCursor) {
    assert(location != null && comment != null, "Bindings_Generator: null source metadata destination");

    if !location.file || clang_equalCursors(cur, clang_getCursorDefinition(cur)) != 0 {
        fill_location(location, cur);
        ge_fill_comment(comment, cur, location.*);
    }
}

// Logs the kind and spelling of every descendant of `cur`, indented by depth.
debug_print_cursor :: (cur: CXCursor) {
    ge_debug_cursor(cur);
}

#scope_file
#import "Basic";
#load "../internal/developer-tools/generator-emission.jai";
