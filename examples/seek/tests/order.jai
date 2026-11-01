// Output is the same whatever the number of threads, and files that are not text or not meant
// to be searched stay out of it.

#import "Basic";
#import "String";
#import "File";
#import "Sort";
#import "Extensions/Testing";
#import,dir "../search";

#import,dir "support";

// Sixty files in nested folders, some lines of each matching, written in an order unrelated to
// their names.
a_big_tree :: (t: *Test) -> string {
    root := tree(t);

    for 0..59 {
        n := (it * 37) % 60;
        folder := ifx n % 3 == 0 then "alpha" else ifx n % 3 == 1 then "beta/inner" else "gamma";
        body := tprint("line zero\nfile % has a needle\nline two\nneedle again % \n", n, n * 2);

        add_file(root, tprint("%/f%.txt", folder, n), body);
    }

    return root;
} @TempAllocates

the_thread_count_does_not_change_the_output :: (t: *Test) {
    root := a_big_tree(t);
    first := options_for(root, "needle");

    first.jobs = 1;

    one := run_search(first);

    defer free_report(*one);

    expect(one.output.count > 0);

    for jobs: string.["2", "3", "8", "16"] {
        options := options_for(root, "needle");

        options.jobs = string_to_int(jobs);

        report := run_search(options);

        expect_equal(report.output, one.output, tprint("-j %", jobs));
        free_report(*report);
    }
} @test

output_comes_out_in_path_order :: (t: *Test) {
    root := a_big_tree(t);
    options := options_for(root, "needle");

    options.files_only = true;
    options.jobs = 8;

    report := run_search(options);

    defer free_report(*report);

    names := split(report.output, "\n",, temp);

    expect_equal(names.count, 61); // sixty names and the empty piece after the last newline

    for 1..59 {
        expect(compare_strings(names[it - 1], names[it]) < 0, tprint("% comes before %", names[it - 1], names[it]));
    }
} @test

counts_agree_across_thread_counts :: (t: *Test) {
    root := a_big_tree(t);

    for jobs: string.["1", "8"] {
        options := options_for(root, "needle");

        options.count = true;
        options.jobs = string_to_int(jobs);

        report := run_search(options);

        lines := split(report.output, "\n",, temp);

        expect_equal(lines.count, 61);
        expect(contains(report.output, "alpha/f0.txt:2\n"));
        free_report(*report);
    }
} @test

more_threads_than_files_is_fine :: (t: *Test) {
    root := a_small_tree(t);
    options := options_for(root, "needle");

    options.jobs = 8;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "a.txt:2:5: Two needle\nb.md:1:1: needle first\nsub/c.txt:1:6: deep needle\n");
} @test

an_empty_tree_has_no_matches :: (t: *Test) {
    root := tree(t);

    require(make_directory_if_it_does_not_exist(root, recursive = true));

    options := options_for(root, "x");

    options.jobs = 4;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(report.output, "");
    expect_equal(exit_code(report), 1);
} @test

binary_files_are_skipped :: (t: *Test) {
    root := tree(t);

    add_file(root, "text.txt", "needle\n");
    add_file(root, "image.png", "\x89PNG\r\n\x1a\n\x00\x00needle");

    // A NUL after the first 8 KiB does not count: the file is still taken for text.
    add_file(root, "late.txt", tprint("%needle\n\x00", join(..repeat_pad(8192), "",, temp)));

    options := options_for(root, "needle");

    options.files_only = true;

    report := run_search(options);

    defer free_report(*report);

    expect_equal(shown(report, root), "late.txt\ntext.txt\n");
} @test

hidden_folders_are_skipped_at_any_depth :: (t: *Test) {
    root := tree(t);

    add_file(root, "a/.cache/x.txt", "needle\n");
    add_file(root, "a/b/y.txt", "needle\n");
    add_file(root, ".git/config", "needle\n");
    add_file(root, ".dotfile", "needle\n");

    options := options_for(root, "needle");

    options.files_only = true;

    report := run_search(options);

    defer free_report(*report);

    // Hidden folders are left out; a hidden file in a folder that is searched is not.
    expect_equal(shown(report, root), ".dotfile\na/b/y.txt\n");
} @test

repeat_pad :: (count: int) -> [] string {
    pieces := NewArray(count, string,, temp);

    for 0..count - 1 pieces[it] = "x";

    return pieces;
}
