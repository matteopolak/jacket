// What a search is asked to do and what it reports.

Options :: struct {
    pattern: string;
    paths: [] string; // files and folders; empty means the current folder
    ignore_case: bool;
    fixed: bool; // the pattern is plain text, not a regular expression
    files_only: bool; // print the names of files that match, nothing else
    count: bool; // print how many lines match in each file
    no_line_number: bool; // leave the line and column out of the output
    globs: [] string; // only files whose name matches one of these (all files when empty)
    jobs: s64 = 1; // worker threads
}

// The outcome of a search. `output` is exactly what to print on standard output; `errors` are
// the messages for standard error.
Report :: struct {
    output: string;
    errors: [] string;
    matched: bool; // whether any line matched
}

free_report :: (report: *Report) {
    free(report.output);

    for report.errors free(it);

    array_free(report.errors);
    report.* = .{};
}

// What seek exits with, the way grep does: 0 when something matched, 1 when nothing did, 2 when
// anything could not be read (even if something matched).
exit_code :: (report: Report) -> s32 {
    if report.errors.count > 0 return 2;

    return ifx report.matched then cast(s32) 0 else 1;
}
