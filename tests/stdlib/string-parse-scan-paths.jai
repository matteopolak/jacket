// String: the parse_* token readers, scan, atof and the path helpers, checked at run time.
#import "Basic";
#import "String";

Fruit :: enum u8 {
    APPLE :: 1;
    PEAR :: 7;
    FIG :: 200;
}

main :: () {
    // parse_token splits on spaces and treats each delimiter as a token of its own.
    line := "  name = (value);";
    token, ok := parse_token(*line);
    assert(ok && token == "name" && line == " = (value);", "parse_token 1: '%' '%'", token, line);
    token, ok = parse_token(*line);
    assert(ok && token == "=");
    token, ok = parse_token(*line);
    assert(ok && token == "(");
    token, ok = parse_token(*line);
    assert(ok && token == "value");
    token, ok = parse_token(*line);
    assert(ok && token == ")");
    token, ok = parse_token(*line);
    assert(ok && token == ";" && line == "");
    blank := "   \t ";
    token, ok = parse_token(*blank);
    assert(!ok && blank == "   \t ", "a failed parse_token restores the line: '%'", blank);
    custom := "a|b c";
    token, ok = parse_token(*custom, delimiters = "|", spaces = "");
    assert(ok && token == "a");
    token, ok = parse_token(*custom, delimiters = "|", spaces = "");
    assert(ok && token == "|");
    token, ok = parse_token(*custom, delimiters = "|", spaces = "");
    assert(ok && token == "b c", "no spaces means spaces are token bytes: '%'", token);

    // match_token consumes the next token whether or not it matched.
    words := "begin END rest";
    token, ok = match_token(*words, "begin");
    assert(ok && token == "begin");
    token, ok = match_token(*words, "end");
    assert(!ok && token == "END" && words == " rest", "a mismatch still consumes: '%'", words);
    words = "END";
    token, ok = match_token_nocase(*words, "end");
    assert(ok && token == "END");
    token, ok = match_token_nocase(*words, "end");
    assert(!ok, "nothing left to match");

    // parse_bool accepts true/false in any letter case.
    flags := "TRUE false True maybe";
    b, bok := parse_bool(*flags);
    assert(bok && b);
    b, bok = parse_bool(*flags);
    assert(bok && !b);
    b, bok = parse_bool(*flags);
    assert(bok && b);
    b, bok = parse_bool(*flags);
    assert(!bok && !b);

    // parse_enum resolves member names, including in a non-s64 enum.
    fruits := "FIG PEAR plum";
    fruit, fok := parse_enum(*fruits, Fruit);
    assert(fok && fruit == .FIG && cast(u8) fruit == 200);
    fruit, fok = parse_enum(*fruits, Fruit);
    assert(fok && fruit == .PEAR);
    fruit, fok = parse_enum(*fruits, Fruit);
    assert(!fok);

    // parse_float and parse_int.
    numbers := "1.5 -2.25e2 7 -9223372036854775808 9223372036854775808 x";
    value, vok := parse_float(*numbers);
    assert(vok && value == 1.5);
    value, vok = parse_float(*numbers);
    assert(vok && value == -225, "parse_float exponent: %", value);
    integer, iok := parse_int(*numbers);
    assert(iok && integer == 7);
    integer, iok = parse_int(*numbers);
    assert(iok && integer == -9223372036854775808, "s64 minimum: %", integer);
    integer, iok = parse_int(*numbers);
    assert(!iok, "s64 overflow fails: %", integer);
    integer, iok = parse_int(*numbers);
    assert(!iok, "not a number");
    empty := "";
    integer, iok = parse_int(*empty);
    assert(!iok && integer == 0);

    // parse_int range-checks the requested type instead of truncating.
    bytes := "255 256 -1 -128 -129";
    byte, bok2 := parse_int(*bytes, u8);
    assert(bok2 && byte == 255);
    byte, bok2 = parse_int(*bytes, u8);
    assert(!bok2, "256 does not fit in u8 (got %)", byte);
    byte, bok2 = parse_int(*bytes, u8);
    assert(!bok2, "-1 does not fit in u8 (got %)", byte);
    small, sok := parse_int(*bytes, s8);
    assert(sok && small == -128);
    small, sok = parse_int(*bytes, s8);
    assert(!sok, "-129 does not fit in s8 (got %)", small);

    // scan: every conversion, literal percent signs, and the failure cases.
    matched, values := scan("%d%%, %i, %f, %b, %s", "-12%, 99, -0.5, false, \"two words\"");
    assert(matched && values.count == 5, "scan: % %", matched, values.count);
    assert((cast(*int) values[0].value_pointer).* == -12);
    assert((cast(*int) values[1].value_pointer).* == 99);
    assert((cast(*float) values[2].value_pointer).* == -0.5);
    assert(!(cast(*bool) values[3].value_pointer).*);
    assert((cast(*string) values[4].value_pointer).* == "two words");
    assert(values[0].type == type_info(int) && values[2].type == type_info(float) && values[4].type == type_info(string));
    matched, values = scan("", "");
    assert(matched && values.count == 0);
    matched, values = scan("", "x");
    assert(!matched);
    matched, values = scan("x", "");
    assert(!matched);
    matched, values = scan("%", "1");
    assert(!matched, "a lone % at the end of the format");
    matched, values = scan("%q", "1");
    assert(!matched, "unknown conversion");
    matched, values = scan("%s", "\"unterminated");
    assert(!matched);
    matched, values = scan("%s", "\"\"");
    assert(matched && values.count == 1 && (cast(*string) values[0].value_pointer).* == "");
    matched, values = scan("%i!", "12");
    assert(!matched, "format left over");
    matched, values = scan("%i", "12!");
    assert(!matched, "text left over");
    matched, values = scan("%b", "maybe");
    assert(!matched);
    matched, values = scan("100%%", "100%");
    assert(matched && values.count == 0);
    matched, values = scan("%%", "x");
    assert(!matched);

    // atof is C's lenient conversion.
    assert(atof("2.5") == 2.5 && atof("  -0.125xyz") == -0.125 && atof("junk") == 0);
    assert(atof("1e3") == 1000);

    // path_decomp and the other path helpers accept both separators.
    path, base, extension, full := path_decomp("dir/sub/name.tar.gz");
    assert(path == "dir/sub/" && base == "name.tar" && extension == "gz" && full == "name.tar.gz", "decomp 1: % % % %", path, base, extension, full);
    path, base, extension, full = path_decomp("C:\\x\\y.txt");
    assert(path == "C:\\x\\" && base == "y" && extension == "txt" && full == "y.txt");
    path, base, extension, full = path_decomp("name");
    assert(path == "" && base == "name" && extension == "" && full == "name");
    path, base, extension, full = path_decomp("dir/");
    assert(path == "dir/" && base == "" && extension == "" && full == "");
    path, base, extension, full = path_decomp(".hidden");
    assert(path == "" && base == ".hidden" && extension == "" && full == ".hidden");
    path, base, extension, full = path_decomp("file.");
    assert(path == "" && base == "file" && extension == "" && full == "file.", "decomp trailing dot: '%'", base);
    path, base, extension, full = path_decomp("a/..");
    assert(path == "a/" && base == ".." && extension == "" && full == "..", "decomp of '..' keeps both dots: '%'", base);
    path, base, extension, full = decomp_filename("..");
    assert(path == "" && base == ".." && extension == "" && full == "..", "decomp_filename '..': '%'", base);
    path, base, extension, full = path_decomp("");
    assert(path == "" && base == "" && extension == "" && full == "");

    assert(path_filename("a/b/c.txt") == "c.txt" && path_filename("a\\b") == "b" && path_filename("a/") == "" && path_filename("x") == "x");
    ext, has_ext := path_extension("archive.tar.gz");
    assert(has_ext && ext == "gz");
    ext, has_ext = path_extension("a.b/c");
    assert(!has_ext && ext == "");
    ext, has_ext = path_extension("dir/.profile");
    assert(!has_ext, "a dot file has no extension");
    ext, has_ext = path_extension("x..y");
    assert(has_ext && ext == "y");
    assert(path_strip_filename("a/b/c.txt") == "a/b/" && path_strip_filename("c.txt") == "" && path_strip_filename("/") == "/");
    assert(path_strip_extension("a/b.c") == "a/b" && path_strip_extension("file.") == "file" && path_strip_extension("..") == "..");
    assert(basename("dir/name.txt") == "name" && path_basename("a\\b\\c") == "c" && basename(".bashrc") == ".bashrc");
    assert(path_ensure_extension("note", "md") == "note.md" && path_ensure_extension("note.md", "md") == "note.md");

    separators := copy_string("a/b\\c/d");
    path_overwrite_separators(separators, #char "/");
    assert(separators == "a/b/c/d");
    path_overwrite_separators(separators, #char "\\");
    assert(separators == "a\\b\\c\\d");
    path_overwrite_separators(separators);
    #if OS == .WINDOWS assert(separators == "a\\b\\c\\d");
    else assert(separators == "a/b/c/d");
    free(separators);

    #if OS == .WINDOWS {
        assert(is_absolute_path("C:\\dir") && is_absolute_path("d:/x") && !is_absolute_path("C:dir") && !is_absolute_path("dir"));
    } else {
        assert(is_absolute_path("/usr") && !is_absolute_path("usr/bin") && !is_absolute_path("") && !is_absolute_path("C:\\dir"));
    }

    print("ok\n");
}
