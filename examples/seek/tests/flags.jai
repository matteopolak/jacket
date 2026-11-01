// Each option of the command line, and the exit status it leads to.

#import "Basic";
#import "String";
#import "Extensions/Testing";
#import,dir "../search";

#import,dir "support";

plain_matches_print_path_line_column_and_text :: (t: *Test) {
    root := a_small_tree(t);
    report := run_search(options_for(root, "needle"));

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt:2:5: Two needle\nb.md:1:1: needle first\nsub/c.txt:1:6: deep needle\n");
    expect(report.matched);
    expect_equal(exit_code(report), 0);
} @test

ignore_case_also_finds_other_spellings :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "NEEDLE");
    options.ignore_case = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt:2:5: Two needle\nb.md:1:1: needle first\nb.md:2:1: Needle again\nsub/c.txt:1:6: deep needle\n");
} @test

without_ignore_case_the_case_matters :: (t: *Test) {
    root := a_small_tree(t);
    report := run_search(options_for(root, "Needle"));

    defer free_report(*report);

    expect_equal(shown(report, root), "b.md:2:1: Needle again\n");
} @test

fixed_text_is_not_a_pattern :: (t: *Test) {
    root := tree(t);

    add_file(root, "dots.txt", "a.b\naxb\n(a)\n");

    plain := options_for(root, "a.b");
    plain.fixed = true;

    report := run_search(plain);

    expect_equal(shown(report, root), "dots.txt:1:1: a.b\n");
    free_report(*report);

    // The same text as a pattern: the dot matches any character.
    report = run_search(options_for(root, "a.b"));
    expect_equal(shown(report, root), "dots.txt:1:1: a.b\ndots.txt:2:1: axb\n");
    free_report(*report);

    // Characters that mean something in a pattern are plain in fixed text.
    brackets := options_for(root, "(a)");
    brackets.fixed = true;

    report = run_search(brackets);
    expect_equal(shown(report, root), "dots.txt:3:1: (a)\n");
    free_report(*report);
} @test

fixed_text_can_ignore_case :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "AGAIN");
    options.fixed = true;
    options.ignore_case = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "b.md:2:8: Needle again\n");
} @test

patterns_use_the_regex_module :: (t: *Test) {
    root := tree(t);

    add_file(root, "log.txt", "error: disk 12 full\nwarn: disk 7 slow\nerror: net down\n");

    report := run_search(options_for(root, "^error: (disk|net) [0-9]*"));

    defer free_report(*report);

    expect_equal(shown(report, root), "log.txt:1:1: error: disk 12 full\nlog.txt:3:1: error: net down\n");
} @test

the_column_counts_characters_not_bytes :: (t: *Test) {
    root := tree(t);

    add_file(root, "u.txt", "héllo wörld needle\n");

    report := run_search(options_for(root, "needle"));

    defer free_report(*report);

    expect_equal(shown(report, root), "u.txt:1:13: héllo wörld needle\n");
} @test

files_only_names_each_file_once :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "needle");
    options.files_only = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt\nb.md\nsub/c.txt\n");
} @test

count_gives_a_number_for_every_text_file :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "needle");
    options.count = true;
    options.ignore_case = true;

    report := run_search(options);

    defer free_report(*report);

    // The binary file and the hidden folder are not mentioned.
    expect_equal(shown(report, root), "a.txt:1\nb.md:2\nsub/c.txt:1\n");
    expect_equal(exit_code(report), 0);
} @test

count_reports_zero_and_the_status_is_one :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "absent");
    options.count = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt:0\nb.md:0\nsub/c.txt:0\n");
    expect_equal(exit_code(report), 1);
} @test

no_line_number_leaves_out_line_and_column :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "deep");
    options.no_line_number = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "sub/c.txt: deep needle\n");
} @test

a_glob_picks_files_by_name :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "needle");
    options.globs = string.["*.md", "c.*"];

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "b.md:1:1: needle first\nsub/c.txt:1:6: deep needle\n");
} @test

a_file_named_on_the_command_line_is_searched_even_in_a_hidden_folder :: (t: *Test) {
    root := a_small_tree(t);

    report := run_search(options_in("needle", tprint("%/.hidden", root)));

    defer free_report(*report);

    expect_equal(shown(report, root), ".hidden/e.txt:1:1: needle in hiding\n");
} @test

several_paths_are_searched_in_path_order_without_repeats :: (t: *Test) {
    root := a_small_tree(t);

    report := run_search(options_in("needle", tprint("%/sub", root), tprint("%/b.md", root), tprint("%/sub/c.txt", root)));

    defer free_report(*report);

    expect_equal(shown(report, root), "b.md:1:1: needle first\nsub/c.txt:1:6: deep needle\n");
} @test

no_match_exits_with_one_and_prints_nothing :: (t: *Test) {
    root := a_small_tree(t);
    report := run_search(options_for(root, "zzz"));

    defer free_report(*report);

    expect_equal(report.output, "");
    expect(!report.matched);
    expect_equal(report.errors.count, 0);
    expect_equal(exit_code(report), 1);
} @test

a_missing_path_exits_with_two_even_when_something_matched :: (t: *Test) {
    root := a_small_tree(t);

    report := run_search(options_in("needle", tprint("%/a.txt", root), tprint("%/nope", root)));

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt:2:5: Two needle\n");
    expect_equal(complaints(report, root), "nope: no such file or folder");
    expect_equal(exit_code(report), 2);
} @test

a_bad_pattern_exits_with_two_before_looking_at_any_file :: (t: *Test) {
    root := a_small_tree(t);
    report := run_search(options_for(root, "(unclosed"));

    defer free_report(*report);

    expect_equal(report.output, "");
    expect_equal(report.errors.count, 1);
    expect(begins_with(report.errors[0], "bad pattern: "));
    expect_equal(exit_code(report), 2);
} @test

windows_line_endings_are_not_part_of_the_text :: (t: *Test) {
    root := tree(t);

    add_file(root, "crlf.txt", "needle one\r\nneedle two\r\n");

    report := run_search(options_for(root, "two$"));

    defer free_report(*report);

    expect_equal(shown(report, root), "crlf.txt:2:8: needle two\n");
} @test

a_last_line_without_a_newline_is_searched :: (t: *Test) {
    root := tree(t);

    add_file(root, "tail.txt", "first\nneedle");

    report := run_search(options_for(root, "needle"));

    defer free_report(*report);

    expect_equal(shown(report, root), "tail.txt:2:1: needle\n");
} @test
