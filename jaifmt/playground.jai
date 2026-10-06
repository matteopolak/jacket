// jaifmt for the browser playground. The page adds this file to the workspace next to the user's
// files and runs it with `engine.play(files, "__jaifmt__.jai")`. It formats `/workspace/<TARGET>`
// with the nearest `jaifmt.toml` between the target's directory and `/workspace`, prints the
// formatted text to stdout and returns 0. On an error it prints `jaifmt: ...` to stderr, prints
// nothing to stdout and returns 1. The page replaces the TARGET line to format another file.
// See docs/tools/jaifmt.md.

TARGET :: "main.jai";

main :: () -> int {
    path := tprint("%/%", WORKSPACE, TARGET);
    source, read_ok := read_entire_file(path, log_errors = false);
    if !read_ok return fail("%: cannot read the file", TARGET);

    config: Format_Config;
    directory := parent_directory(path);
    while true {
        toml, has_config := read_entire_file(tprint("%/%", directory, FORMAT_CONFIG_FILE_NAME), log_errors = false);
        if has_config {
            parsed, ok, error := parse_config(toml);
            if !ok return fail("%/%: %", directory, FORMAT_CONFIG_FILE_NAME, error);
            config = parsed;
            break;
        }
        if directory == WORKSPACE break;
        directory = parent_directory(directory);
    }

    formatted, ok, error := format_source(source, config);
    if !ok return fail("%:%", TARGET, error);
    write_string(formatted);
    return 0;
}

#scope_file

#import "Basic";
#import "File";
#import "Jai_Format";

WORKSPACE :: "/workspace";

fail :: (format: string, args: ..Any) -> int {
    print("jaifmt: %\n", tprint(format, ..args), to_standard_error = true);
    return 1;
}

parent_directory :: (path: string) -> string {
    for < i: 0..path.count - 1 {
        if path[i] == #char "/" return ifx i == 0 then "/" else string.{ i, path.data };
    }
    return WORKSPACE;
}
