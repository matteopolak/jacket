// String: every search, split, trim, case and edit helper, checked at run time with exact
// results, including empty inputs, non-ASCII UTF-8 bytes, delimiters at both ends and
// not-found cases.
#import "Basic";
#import "String";

main :: () {
    // compare / equal, plain and case-folded.
    assert(compare("abc", "abd") == -1 && compare("abd", "abc") == 1 && compare("abc", "abc") == 0);
    assert(compare("ab", "abc") == -1 && compare("abc", "ab") == 1 && compare("", "") == 0);
    assert(compare("", "a") == -1, "empty sorts first");
    assert(compare("\xff", "a") == 1, "bytes compare unsigned");
    assert(compare_nocase("HELLO", "hello") == 0 && compare_nocase("Apple", "banana") == -1);
    assert(compare_nocase("b", "A") == 1 && compare_nocase("abc", "ABCD") == -1);
    assert(equal("x", "x") && !equal("x", "X") && equal("", ""));
    assert(equal_nocase("MiXeD", "mixed") && !equal_nocase("mixed", "mixe") && equal_nocase("", ""));
    assert(!equal_nocase("é", "É"), "case folding is ASCII only");

    // Prefix and suffix tests.
    assert(begins_with("filename.txt", "file") && !begins_with("fi", "file") && begins_with("x", ""));
    assert(ends_with("filename.txt", ".txt") && !ends_with("txt", ".txt") && ends_with("", ""));
    assert(starts_with("abc", "ab"));
    assert(begins_with_nocase("README.md", "readme") && !begins_with_nocase("READ", "readme"));
    assert(ends_with_nocase("photo.JPG", ".jpg") && !ends_with_nocase("photo.png", ".jpg"));
    assert(begins_with_nocase("", "") && ends_with_nocase("abc", ""));

    // Byte and substring searches.
    s := "abcabc";
    assert(find_index_from_left(s, #char "b") == 1 && find_index_from_right(s, #char "b") == 4);
    assert(find_index_from_left(s, #char "b", 2) == 4 && find_index_from_left(s, #char "b", 5) == -1);
    assert(find_index_from_left(s, #char "a", -3) == 0, "a negative start clamps to 0");
    assert(find_index_from_left(s, #char "z") == -1 && find_index_from_right(s, #char "z") == -1);
    assert(find_index_from_left("", #char "a") == -1 && find_index_from_right("", #char "a") == -1);
    assert(find_index_from_left(s, "bc") == 1 && find_index_from_right(s, "bc") == 4);
    assert(find_index_from_left(s, "bc", 2) == 4 && find_index_from_left(s, "abcabcd") == -1);
    assert(find_index_from_right(s, "abc") == 3 && find_index_from_right(s, "") == -1);
    assert(find_index_from_left("aaa", "aa") == 0 && find_index_from_right("aaa", "aa") == 1);
    assert(find_index_of_any_from_left("hello, world", " ,") == 5);
    assert(find_index_of_any_from_right("hello, world", " ,") == 6);
    assert(find_index_of_any_from_left("hello, world", ",", 6) == -1);
    assert(find_index_of_any_from_left("abc", "") == -1 && find_index_of_any_from_right("abc", "xyz") == -1);
    assert(is_any(#char "b", "abc") && !is_any(#char "d", "abc") && !is_any(#char "a", ""));

    index, found := index_of_string_nocase("Hello World", "WORLD");
    assert(found && index == 6, "index_of_string_nocase: % %", index, found);
    index, found = index_of_string_nocase("Hello", "HELLO!");
    assert(!found && index == -1);
    index, found = index_of_string_nocase("xxAbAB", "ab");
    assert(found && index == 2);

    // contains and its variants return what follows the match.
    ok, rest := contains("key: value", ": ");
    assert(ok && rest == "value");
    ok, rest = contains("key: value", "=");
    assert(!ok && rest == "");
    ok, rest = contains("a.b.c", #char ".");
    assert(ok && rest == "b.c");
    ok, rest = contains_nocase("Content-TYPE: text", "content-type:");
    assert(ok && rest == " text", "contains_nocase: %", rest);
    ok, rest = contains_nocase("abc", "abcd");
    assert(!ok && rest == "");
    ok, rest = contains_any_character("path/to\\file", "\\/");
    assert(ok && rest == "to\\file", "contains_any_character: %", rest);
    ok, rest = contains_any_character("plain", "/\\");
    assert(!ok && rest == "");
    ok, rest = contains_any_string("one two three", "four", "two", "one");
    assert(ok && rest == " three", "contains_any_string tries the needles in order: %", rest);
    ok, rest = contains_any_string("one two three", "four", "five");
    assert(!ok && rest == "");
    ok, rest = contains_any_string("anything");
    assert(!ok, "no needles");
    ok, rest = contains("trailing=", #char "=");
    assert(ok && rest == "", "a match at the end leaves nothing");

    // replace_chars edits in place.
    edited := copy_string("a-b_c-d");
    replace_chars(edited, "-_", #char " ");
    assert(edited == "a b c d", "replace_chars: %", edited);
    replace_chars(edited, "", #char "!");
    assert(edited == "a b c d");
    free(edited);

    // split_from_left/right by byte, by string and by any byte.
    f, l, r := split_from_left("k=v=w", #char "=");
    assert(f && l == "k" && r == "v=w");
    f, l, r = split_from_right("k=v=w", #char "=");
    assert(f && l == "k=v" && r == "w");
    f, l, r = split_from_left("none", #char "=");
    assert(!f && l == "none" && r == "", "split_from_left keeps the whole text on the left");
    f, l, r = split_from_right("none", #char "=");
    assert(!f && l == "" && r == "none", "split_from_right keeps the whole text on the right");
    f, l, r = split_from_left("=edge=", #char "=");
    assert(f && l == "" && r == "edge=");
    f, l, r = split_from_right("=edge=", #char "=");
    assert(f && l == "=edge" && r == "");
    f, l, r = split_from_left("a::b::c", "::");
    assert(f && l == "a" && r == "b::c");
    f, l, r = split_from_right("a::b::c", "::");
    assert(f && l == "a::b" && r == "c");
    f, l, r = split_from_right("abc", "::");
    assert(!f && l == "" && r == "abc");
    f, l, r = split_from_right("", "::");
    assert(!f && l == "" && r == "");
    f, l, r = split_from_left_by_any("a b,c", ", ");
    assert(f && l == "a" && r == "b,c");
    f, l, r = split_from_right_by_any("a b,c", ", ");
    assert(f && l == "a b" && r == "c");
    f, l, r = split_from_left_by_any("abc", ",;");
    assert(!f && l == "abc" && r == "");
    f, l, r = split_from_right_by_any("abc", ",;");
    assert(!f && l == "" && r == "abc");

    // split keeps empty pieces, including leading and trailing ones; join inverts it.
    pieces := split(",a,,b,", #char ",");
    assert(pieces.count == 5, "split count %", pieces.count);
    expected := string.["", "a", "", "b", ""];
    for expected assert(pieces[it_index] == it, "piece % is '%'", it_index, pieces[it_index]);
    joined := join(..pieces, separator = ",");
    assert(joined == ",a,,b,", "join(split(x)) == x: %", joined);
    free(joined);
    array_free(pieces);
    pieces = split("one<>two<>three", "<>");
    assert(pieces.count == 3 && pieces[0] == "one" && pieces[1] == "two" && pieces[2] == "three");
    array_free(pieces);
    pieces = split("", #char ",");
    assert(pieces.count == 1 && pieces[0] == "", "split of empty text: %", pieces.count);
    array_free(pieces);
    pieces = split("no separators", #char ",");
    assert(pieces.count == 1 && pieces[0] == "no separators");
    array_free(pieces);
    pieces = split("héllo wörld", #char " ");
    assert(pieces.count == 2 && pieces[0] == "héllo" && pieces[1] == "wörld", "UTF-8 pieces");
    array_free(pieces);

    joined = join("a", "", "b", separator = "--");
    assert(joined == "a----b", "join with an empty part: %", joined);
    free(joined);
    joined = join("only");
    assert(joined == "only");
    free(joined);
    joined = join("x", separator = ", ", before_first = true);
    assert(joined == ", x", "before_first: %", joined);
    free(joined);
    assert(join("", "", separator = "") == "", "an empty total joins to empty");

    // Long input: split and join 1000 parts.
    long_builder: String_Builder;
    for 0..999 {
        if it append(*long_builder, ";");
        print_to_builder(*long_builder, "%", it);
    }
    long_text := builder_to_string(*long_builder);
    pieces = split(long_text, #char ";");
    assert(pieces.count == 1000 && pieces[999] == "999" && pieces[500] == "500");
    rejoined := join(..pieces, separator = ";");
    assert(rejoined == long_text);
    free(rejoined);
    array_free(pieces);
    free(long_text);

    // Trimming.
    assert(trim("  \t padded \r\n") == "padded" && trim("") == "" && trim("   ") == "");
    assert(trim_left("  x  ") == "x  " && trim_right("  x  ") == "  x");
    assert(trim("xxhixx", "x") == "hi" && trim_left("--a", "-") == "a" && trim_right("a--", "-") == "a");
    assert(trim("no-op", "") == "no-op");

    // stop_at_any and the eat_ helpers.
    assert(stop_at_any("name=value;rest", "=;") == "name");
    assert(stop_at_any("plain", "=;") == "plain" && stop_at_any("", "=") == "");
    assert(stop_at_any("=first", "=") == "");
    assert(eat_trailing_spaces("text \t \t") == "text" && eat_trailing_spaces("   ") == "");
    assert(eat_trailing_spaces("keep\n") == "keep\n", "only spaces and tabs are eaten");
    assert(eat_until_space("word rest") == " rest" && eat_until_space("word\tx") == "\tx");
    assert(eat_until_space("nospace") == "" && eat_until_space(" lead") == " lead");

    // Case conversion: in place on writable strings, copies leave the input alone.
    writable := copy_string("Hello, Wörld 123");
    to_upper_in_place(writable);
    assert(writable == "HELLO, WöRLD 123", "to_upper_in_place: %", writable);
    to_lower_in_place(writable);
    assert(writable == "hello, wörld 123", "to_lower_in_place: %", writable);
    free(writable);
    source := "MiXeD-Case_42";
    lower := to_lower_copy(source);
    upper := to_upper_copy(source);
    assert(lower == "mixed-case_42" && upper == "MIXED-CASE_42" && source == "MiXeD-Case_42");
    assert(lower.data != source.data, "the copy is a new buffer");
    free(lower);
    free(upper);
    empty_lower := to_lower_copy("");
    assert(empty_lower == "");
    free(empty_lower);

    // replace: counts, overlapping candidates, growing and shrinking, and empty patterns.
    replaced, count := replace("aaaa", "aa", "b");
    assert(replaced == "bb" && count == 2, "replace non-overlapping: % %", replaced, count);
    free(replaced);
    replaced, count = replace("abc", "x", "y");
    assert(replaced == "abc" && count == 0);
    free(replaced);
    replaced, count = replace("abc", "", "y");
    assert(replaced == "abc" && count == 0, "an empty pattern replaces nothing");
    free(replaced);
    replaced, count = replace("a.b.c", ".", "::");
    assert(replaced == "a::b::c" && count == 2);
    free(replaced);
    replaced, count = replace("xx", "x", "");
    assert(replaced == "" && count == 2);
    free(replaced);
    replaced, count = replace("ünï", "ï", "i");
    assert(replaced == "üni" && count == 1, "UTF-8 bytes replace like any other: %", replaced);
    free(replaced);

    // slice clamps to the end of the string.
    assert(slice("hello", 1, 3) == "ell" && slice("hello", 3, 100) == "lo" && slice("hello", 5, 1) == "");
    assert(slice("hello", 9, 1) == "" && slice("", 0, 0) == "");

    // CRLF normalisation, in place and as a copy.
    text := copy_string("\r\n\r\nx\r");
    normalize_line_endings(*text);
    assert(text == "\n\nx\r");
    free(text);
    normalized := normalize_line_endings("");
    assert(normalized == "");
    free(normalized);

    // Wildcards.
    assert(wildcard_match("", "") && wildcard_match("", "*") && !wildcard_match("", "?"));
    assert(wildcard_match("abc", "a*") && wildcard_match("abc", "*c") && wildcard_match("abc", "a*b*c"));
    assert(wildcard_match("abc", "???") && !wildcard_match("abc", "????") && !wildcard_match("abc", "ab"));
    assert(wildcard_match("main.jai", "*.jai") && !wildcard_match("main.jai.bak", "*.jai"));
    assert(wildcard_match("b", "[a-c]") && wildcard_match("b", "[c-a]") && !wildcard_match("d", "[a-c]"));
    assert(wildcard_match("x", "[xyz]") && !wildcard_match("w", "[xyz]"));
    assert(wildcard_match("aXbXc", "*X*X*") && !wildcard_match("aXb", "*X*X*"));

    print("ok\n");
}
