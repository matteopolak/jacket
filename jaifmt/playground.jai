// jaifmt for the browser playground. The page adds this file to the workspace next to the user's
// files and runs it with `engine.play(files, "__jaifmt__.jai")`. It formats `/workspace/<TARGET>`
// with the nearest `jacket.toml` (its `[format]` table) between the target's directory and
// `/workspace`, prints the formatted text to stdout and returns 0. On an error it
// prints `jaifmt: ...` to stderr, prints nothing to stdout and returns 1. The page replaces the
// TARGET line to format another file. See docs/tools/jaifmt.md.

TARGET :: "main.jai";

main :: () -> int {
    path := tprint("%/%", WORKSPACE, TARGET);
    source, read_ok := read_entire_file(path, log_errors = false);

    if !read_ok return fail("%: cannot read the file", TARGET);

    directory := format_parent_directory(path);
    config, file, config_ok, config_error := format_find_config(directory, read_config_file, WORKSPACE);

    if !config_ok return fail("%: %", file, config_error);

    formatted, ok, error := format_source(source, config);
    if !ok return fail("%:%", TARGET, error);

    write_string(formatted);

    return 0;
}

#scope_file

#import "Basic";
#import "File";
#import "Extensions/Jai_Format";

WORKSPACE :: "/workspace";

fail :: (format: string, args: ..Any) -> int {
    print("jaifmt: %\n", tprint(format, ..args), to_standard_error = true);
    return 1;
}

read_config_file :: (path: string) -> string, bool {
    return read_entire_file(path, log_errors = false);
}
