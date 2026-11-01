// The small parts: globs, the matcher and the binary check.

#import "String";
#import "Extensions/Testing";
#import,dir "../search";

globs_match_names_and_paths :: (t: *Test) {
    expect(glob_matches("*.jai", "main.jai"));
    expect(glob_matches("*.jai", "dir/sub/main.jai"));
    expect(!glob_matches("*.jai", "main.jai.bak"));
    expect(glob_matches("main.*", "x/main.c"));
    expect(glob_matches("m?in.c", "main.c"));
    expect(!glob_matches("m?in.c", "min.c"));
    expect(glob_matches("*", "anything"));
    expect(glob_matches("", ""));
    expect(!glob_matches("", "a"));
} @test

globs_with_a_slash_match_the_whole_path :: (t: *Test) {
    expect(glob_matches("src/*.jai", "src/a.jai"));
    expect(glob_matches("src/*.jai", "./src/a.jai"));
    expect(!glob_matches("src/*.jai", "src/deep/a.jai"));
    expect(glob_matches("src/**.jai", "src/deep/er/a.jai"));
    expect(!glob_matches("lib/*.jai", "src/a.jai"));
    expect(glob_matches("*/a.jai", "x/a.jai"));
} @test

globs_have_character_sets :: (t: *Test) {
    expect(glob_matches("[abc].txt", "b.txt"));
    expect(!glob_matches("[abc].txt", "d.txt"));
    expect(glob_matches("file[0-9].log", "file7.log"));
    expect(!glob_matches("file[0-9].log", "fileX.log"));
    expect(glob_matches("[!a]x", "bx"));
    expect(!glob_matches("[!a]x", "ax"));
    expect(glob_matches("a[]]b", "a]b"));
    expect(glob_matches("a[b", "a[b")); // no closing bracket: a plain [
} @test

an_empty_list_of_globs_picks_everything :: (t: *Test) {
    expect(glob_any(.[], "whatever.bin"));
    expect(glob_any(string.["*.md", "*.txt"], "notes.txt"));
    expect(!glob_any(string.["*.md", "*.txt"], "notes.rs"));
} @test

only_a_nul_in_the_first_8_kib_makes_a_file_binary :: (t: *Test) {
    expect(!looks_binary(""));
    expect(!looks_binary("text\nwith lines\n"));
    expect(looks_binary("\x00"));
    expect(looks_binary("abc\x00def"));
} @test

a_fixed_matcher_finds_the_first_place :: (t: *Test) {
    matcher, problem := make_matcher(.{ pattern = "na", fixed = true });

    defer free_matcher(*matcher);

    expect_equal(problem, "");
    expect_equal(find_in_line(*matcher, "banana"), 2);
    expect_equal(find_in_line(*matcher, "apple"), -1);
    expect_equal(find_in_line(*matcher, ""), -1);
} @test

an_empty_fixed_pattern_matches_every_line :: (t: *Test) {
    matcher := make_matcher(.{ pattern = "", fixed = true });

    defer free_matcher(*matcher);

    expect_equal(find_in_line(*matcher, "abc"), 0);
    expect_equal(find_in_line(*matcher, ""), 0);
} @test

a_case_folding_matcher_reports_offsets_in_the_original :: (t: *Test) {
    matcher := make_matcher(.{ pattern = "Cd", fixed = true, ignore_case = true });

    defer free_matcher(*matcher);

    expect_equal(find_in_line(*matcher, "ABCDEF"), 2);
    expect_equal(find_in_line(*matcher, "abcdef"), 2);
    expect_equal(find_in_line(*matcher, "xyz"), -1);
} @test

a_regex_matcher_finds_the_first_match :: (t: *Test) {
    matcher, problem := make_matcher(.{ pattern = "[0-9]+" });

    defer free_matcher(*matcher);

    expect_equal(problem, "");
    expect_equal(find_in_line(*matcher, "room 101"), 5);
    expect_equal(find_in_line(*matcher, "no digits"), -1);
} @test

a_bad_regex_is_explained :: (t: *Test) {
    _, problem := make_matcher(.{ pattern = "a(" });

    expect(begins_with(problem, "bad pattern: "), problem);
} @test
