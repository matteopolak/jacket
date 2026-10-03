string_tests :: () {
    assert(compare("a", "b") < 0 && compare("b", "a") > 0);
    assert(compare("ab", "ab") == 0 && compare("a", "ab") < 0);
    assert(equal_nocase("MiXeD", "mixed"));
    assert(begins_with("word", "") && ends_with("word", ""));
    assert(find_index_from_left("banana", "ana") == 1);
    assert(find_index_from_right("banana", "ana") == 3);
    assert(find_index_from_left("word", "") == -1);
    found, remainder := contains("abcabc", "bc");

    assert(found && remainder == "abc");
    assert(slice("abc", 1, 99) == "bc" && slice("abc", 3, 1).count == 0);
    assert(trim(" \tvalue\r\n") == "value");
    fields := split("|a||", cast(u8) 124);

    defer array_free(fields);
    assert(fields.count == 4 && fields[0] == "" && fields[1] == "a" && fields[2] == "" && fields[3] == "");
    replaced, count := replace("aaaaa", "aa", "x");

    defer free(replaced);
    assert(replaced == "xxa" && count == 2);
    normalized := normalize_line_endings("a\r\nb\rc");

    defer free(normalized);
    assert(normalized == "a\nb\rc");
    assert(path_filename("dir/name.ext") == "name.ext");
    assert(path_strip_filename("dir/name.ext") == "dir/");
    assert(path_strip_extension("dir/name.ext") == "dir/name");
    extension, has_extension := path_extension(".hidden");

    assert(!has_extension && extension.count == 0);
    line := " \ttrue;word";
    value, valid := parse_bool(*line);

    assert(valid && value && line == ";word");
    token, token_ok := parse_token(*line);

    assert(token_ok && token == ";" && line == "word");
    assert(wildcard_match("abc", "[a-c]*?") && !wildcard_match("a", "??"));
}
