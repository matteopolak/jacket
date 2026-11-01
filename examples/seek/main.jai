// seek: a small grep. Searches files and folders for a pattern, in parallel, and prints
// `path:line:col: text` for every line that matches, in path order. README.md is the guide.
//
//     jaic examples/seek/main.jai -exe seek
//     ./seek needle examples
//     ./seek -i --glob '*.jai' 'fn(ish)?' src
//     ./seek -c -F 'a.b' notes.txt
//
// Exit status: 0 when a line matched, 1 when none did, 2 on a bad command line or an unreadable
// file.

#import "Basic";
#import "System";
Args :: #import "Extensions/Args";
#import,dir "search";

Cli :: struct {
    pattern: string; @positional @required
    paths: [..] string; @positional
    ignore_case: bool; @short=i
    fixed: bool; @short=F
    files_only: bool; @short=l
    count: bool; @short=c
    no_line_number: bool; @short=n
    glob: [..] string; @short=g @value=PATTERN
    jobs: s64; @short=j @value=N @range(0,256)
}

HELP :: Args.Help(Cli).{
    about = "Searches files for a pattern and prints path:line:col: text for each matching line.",
    version = "0.1.0",
    pattern = "What to look for: a regular expression, or plain text with -F",
    paths = "Files and folders to search (default: the current folder)",
    ignore_case = "Ignore case",
    fixed = "Treat the pattern as plain text (-i then folds ASCII letters only)",
    files_only = "Print only the names of files with a match",
    count = "Print only how many lines match in each file",
    no_line_number = "Leave the line and column out: path: text",
    glob = "Only search files whose name matches this pattern (* ? [a-z]; repeatable)",
    jobs = "Worker threads (default: one per processor)",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    options := Options.{
        pattern = cli.pattern,
        paths = cli.paths,
        ignore_case = cli.ignore_case,
        fixed = cli.fixed,
        files_only = cli.files_only,
        count = cli.count,
        no_line_number = cli.no_line_number,
        globs = cli.glob,
        jobs = ifx cli.jobs > 0 then cli.jobs else get_number_of_processors(),
    };

    report := run_search(options);

    write_string(report.output);

    for report.errors print("seek: %\n", it, to_standard_error = true);

    exit(exit_code(report));
}
