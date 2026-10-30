#import "Basic";
#import "Extensions/Testing";

Choice :: enum {
    FIRST;
    SECOND;
}

contains_nocase_returns_the_text_after_the_match :: () {
    text := "abcDEFghi";
    found, remainder := S.contains_nocase(text, "def");

    assert_that(found, is_true());
    assert_that(remainder, is("ghi"));
    expect(remainder.data == text.data + 6);
    absent, empty := S.contains(text, "");

    assert_that(absent, is_false());
    assert_that(empty, is_empty());
    index, indexed := S.index_of_string_nocase(text, "DEF");

    expect(indexed && index == 3);
    expect(S.begins_with(text, "") && S.ends_with_nocase(text, ""));
} @test

split_from_right_returns_all_of_the_text_first_when_nothing_matches :: () {
    missing, left, right := S.split_from_right("abc", cast(u8) 124);

    expect(!missing && left == "abc" && right == "");
    missing, left, right = S.split_from_right("abc", "|");
    expect(!missing && left == "abc" && right == "");
    missing, left, right = S.split_from_right_by_any("abc", "|;");
    expect(!missing && left == "abc" && right == "");
} @test

splitting_and_replacing_with_an_empty_pattern :: () {
    fields := S.split("", "");

    defer array_free(fields);
    expect(fields.count == 1 && fields[0].count == 0);
    replacement, count := S.replace("aaa", "", "X");

    defer free(replacement);
    expect(replacement == "aaa" && count == 0);
} @test

line_endings_and_case_are_changed_in_place :: () {
    writable := copy_string("A\r\nB\rC");
    original := writable.data;

    defer free(original);
    S.normalize_line_endings(*writable);
    expect(writable == "A\nB\rC" && writable.data == original);
    S.to_lower_in_place(writable);
    expect(writable == "a\nb\rc");
} @test

path_helpers_keep_or_trim_the_edges :: () {
    joined := S.path_join("a//", "b/", separator = cast(u8) 47);

    defer free(joined);
    expect(joined == "a//b");
    path := "dir/file.ext";
    same_path := S.path_ensure_extension(path, "ext");

    expect(same_path.data == path.data && same_path.count == path.count);
    expect(S.path_filename("dir/") == "" && S.path_strip_filename("file") == "");
} @test

token_parsing_leaves_the_line_as_documented :: () {
    line := " \t";
    saved := line;
    token, ok := S.parse_token(*line);

    expect(!ok && token.count == 0 && line.data == saved.data && line.count == saved.count);
    line = "  wrong;tail";
    token, ok = S.match_token(*line, "right");
    expect(!ok && token == "wrong" && line == ";tail");
    line = " 257;rest";
    narrow, parsed := S.parse_int(*line, u8);

    expect(!parsed && narrow == 0 && line == ";rest");
    line = "SECOND;rest";
    choice, enum_parsed := S.parse_enum(*line, Choice);

    expect(enum_parsed && choice == .SECOND && line == ";rest");
} @test

scan_stops_at_the_first_mismatch :: () {
    scanned, values := S.scan("%d X", "12 Y");

    expect(!scanned && values.count == 1 && values[0].type == type_info(int));
    expect_equal((cast(*int) values[0].value_pointer).*, 12);
    scanned, values = S.scan("%%", "%");
    expect(scanned && values.count == 0);
    scanned, values = S.scan("%d%%", "50%");
    expect(scanned && values.count == 1);
    scanned, values = S.scan("%%", "%x");
    expect(!scanned);
    expect(S.scan2("%", "%%"));
    first: int = -1;
    second: int = -2;

    expect(!S.scan2("7 bad", "%d %d", *first, *second));
    expect(first == 7 && second == -2);
    expect(S.scan2("9", "%d", *first, *second) && first == 9 && second == -2);
} @test

wildcards_and_negative_start_offsets :: () {
    expect(S.wildcard_match("b5", "[a-c][0-9]") && !S.wildcard_match("d5", "[a-c][0-9]"));
    expect_equal(S.find_index_from_left("abc", cast(u8) 98, -5), 1);
    expect(S.find_index_of_any_from_left("abc", "c", -5) == 2 && S.find_index_of_any_from_left("", "c", -1) == -1);
} @test

#scope_file

S :: #import "String";
