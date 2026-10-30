// Not-found results of the splits differ by overload, `scan` consumes a literal `%%`, and substring
// search finds matches from any start offset.
#import "Extensions/Testing";

not_found_results_differ_by_overload :: () {
    found, left, right := Str.split_from_right("abc", cast(u8) 124);

    expect(!found && left == "" && right == "abc");
    found, left, right = Str.split_from_right("abc", "|");
    expect(!found && left == "abc" && right == "");
    found, left, right = Str.split_from_left("abc", cast(u8) 124);
    expect(!found && left == "abc" && right == "");
    found, left, right = Str.split_from_right("a|b|c", cast(u8) 124);
    expect(found && left == "a|b" && right == "c");
} @test

substring_search_honors_the_start_offset :: () {
    expect_equal(Str.find_index_from_left("abcabc", "bc"), 1);
    expect_equal(Str.find_index_from_left("abcabc", "bc", 2), 4);
    expect_equal(Str.find_index_from_left("abcabc", "cx"), -1);
    expect_equal(Str.find_index_from_left("abcabc", cast(u8) 99, -5), 2);
} @test

scan_consumes_a_literal_percent :: () {
    ok, values := Str.scan("%%", "%");

    expect(ok && values.count == 0);
    ok, values = Str.scan("%%", "x");
    expect(!ok);
    ok, values = Str.scan("%%", "%%");
    expect(!ok);

    expect(Str.scan2("%", "%%"));
} @test

#scope_file

Str :: #import "String";
