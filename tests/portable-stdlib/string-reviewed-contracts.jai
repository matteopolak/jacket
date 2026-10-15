#import "Basic";
S :: #import "String";

Choice :: enum {
    FIRST;
    SECOND;
}

main :: () {
    text := "abcDEFghi";
    found, remainder := S.contains_nocase(text, "def");

    assert(found && remainder == "ghi" && remainder.data == text.data + 6);
    absent, empty := S.contains(text, "");

    assert(!absent && empty.count == 0);
    index, indexed := S.index_of_string_nocase(text, "DEF");

    assert(indexed && index == 3);
    assert(S.begins_with(text, "") && S.ends_with_nocase(text, ""));
    missing, left, right := S.split_from_right("abc", cast(u8) 124);

    assert(!missing && left == "" && right == "abc");
    missing, left, right = S.split_from_right("abc", "|");
    assert(!missing && left == "abc" && right == "");
    missing, left, right = S.split_from_right_by_any("abc", "|;");
    assert(!missing && left == "abc" && right == "");
    fields := S.split("", "");

    defer array_free(fields);
    assert(fields.count == 1 && fields[0].count == 0);
    replacement, count := S.replace("aaa", "", "X");

    defer free(replacement);
    assert(replacement == "aaa" && count == 0);
    writable := copy_string("A\r\nB\rC");
    original := writable.data;

    defer free(original);
    S.normalize_line_endings(*writable);
    assert(writable == "A\nB\rC" && writable.data == original);
    S.to_lower_in_place(writable);
    assert(writable == "a\nb\rc");
    joined := S.path_join("a//", "b/", separator = cast(u8) 47);

    defer free(joined);
    assert(joined == "a//b");
    path := "dir/file.ext";
    same_path := S.path_ensure_extension(path, "ext");

    assert(same_path.data == path.data && same_path.count == path.count);
    assert(S.path_filename("dir/") == "" && S.path_strip_filename("file") == "");
    line := " \t";
    saved := line;
    token, ok := S.parse_token(*line);

    assert(!ok && token.count == 0 && line.data == saved.data && line.count == saved.count);
    line = "  wrong;tail";
    token, ok = S.match_token(*line, "right");
    assert(!ok && token == "wrong" && line == ";tail");
    line = " 257;rest";
    narrow, parsed := S.parse_int(*line, u8);

    assert(parsed && narrow == 1 && line == ";rest");
    line = "SECOND;rest";
    choice, enum_parsed := S.parse_enum(*line, Choice);

    assert(enum_parsed && choice == .SECOND && line == ";rest");
    scanned, values := S.scan("%d X", "12 Y");

    assert(!scanned && values.count == 1 && values[0].type == type_info(int));
    assert((cast(*int) values[0].value_pointer).* == 12);
    scanned, values = S.scan("%%", "%");
    assert(!scanned && values.count == 0);
    assert(S.scan2("%", "%%"));
    first: int = -1;
    second: int = -2;

    assert(!S.scan2("7 bad", "%d %d", *first, *second));
    assert(first == 7 && second == -2);
    assert(S.scan2("9", "%d", *first, *second) && first == 9 && second == -2);
    assert(S.wildcard_match("b5", "[a-c][0-9]") && !S.wildcard_match("d5", "[a-c][0-9]"));
    print("ok\n");
}
