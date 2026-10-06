#import "Basic";
#import "Math";
#import "Sloppy_Math";
#import "Jai_Lexer";
#import "File_Watcher";
#import "Print_Vars";
#import "String";

// Type extremes, array_insert_at, struct-print indentation, matrix basis builders,
// Jai_Lexer string literals and input reset, File_Watcher change merging, Print_Vars columns.

on_change :: (watcher: *File_Watcher(void), change: *File_Change, user_data: *void) {}

Inner :: struct {
    a: int;
}
Outer :: struct {
    inner: Inner;
}

main :: () -> int {
    if max_of_type(u8) != 255 || max_of_type(s8) != 127 || max_of_type(u64) != 0xffff_ffff_ffff_ffff || max_of_type(s64) != 0x7fff_ffff_ffff_ffff return 1;
    if min_of_type(s8) != -128 || min_of_type(s32) != -2147483648 || min_of_type(u16) != 0 || !max_of_type(bool) return 2;
    if max_of_type(float32) != FLOAT32_MAX || min_of_type(float64) != FLOAT64_MIN return 3;

    xs: [..] int;
    array_insert_at(*xs, 2, 0);
    array_insert_at(*xs, 0, 0);
    array_insert_at(*xs, 3, 2);
    array_insert_at(*xs, 1, 1);
    if xs.count != 4 return 10;
    for xs if it != it_index return 11;

    // Multi-line struct printing indents nested members by depth * indentation_width.
    context.print_style.default_format_struct.use_newlines_if_long_form = true;
    context.print_style.default_format_struct.use_long_form_if_more_than_this_many_members = -1;
    context.print_style.default_format_struct.indentation_width = 3;
    printed := tprint("%", Outer.{ .{ 7 } });
    if !contains(printed, "\n      a = 7") return 12;

    rows := make_matrix_from_rows(.{ 1, 2, 3 }, .{ 4, 5, 6 }, .{ 7, 8, 9 }, Matrix3);
    cols := make_matrix_from_columns(.{ 1, 2, 3 }, .{ 4, 5, 6 }, .{ 7, 8, 9 });
    if rows._12 != 2 || rows._31 != 7 || cols._12 != 4 || cols._31 != 3 || cols._44 != 1 || cols._14 != 0 return 20;

    lexer: Lexer;
    set_input_from_string(*lexer, "s := \"q\\n\";");
    t := peek_next_token(*lexer);
    if t.type != .IDENT return 30;
    for 1..3 eat_token(*lexer); // s, :, =
    t = peek_next_token(*lexer);
    if t.type != .STRING || t.string_value != "q\n" return 31;
    set_input_from_string(*lexer, "\n  x");
    t = peek_next_token(*lexer);
    if t.type != .IDENT || t.l0 != 2 || t.c0 != 3 return 32;

    w: File_Watcher(void);
    if !init(*w, on_change, events_to_watch = .ADDED | .MODIFIED) return 40;
    add_change(*w, "a", .ADDED);
    add_change(*w, "b", .REMOVED);
    add_change(*w, "a", .MODIFIED | .REMOVED);
    if w.changes.count != 1 || w.changes[0].full_path != "a" || w.changes[0].events != (.ADDED | .MODIFIED) return 41;

    builder: String_Builder;
    config: Print_Vars_Config;
    config.name_indentation = .RIGHT_JUSTIFIED;
    print_helper(*builder, config, 5, 4, "ab", "12", int);
    if builder_to_string(*builder) != "   ab = 12   (s64)\n" return 50;
    return 0;
}
