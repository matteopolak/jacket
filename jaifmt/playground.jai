// jaifmt for the browser playground. The page adds this file to the workspace next to the user's
// files and runs it with `engine.play(files, "__jaifmt__.jai")`. It formats `/workspace/<TARGET>`
// with the nearest `jacket.toml` (its `[format]` table) or `jaifmt.toml` between the target's
// directory and `/workspace`, prints the formatted text to stdout and returns 0. On an error it
// prints `jaifmt: ...` to stderr, prints nothing to stdout and returns 1. The page replaces the
// TARGET line to format another file. See docs/tools/jaifmt.md.

TARGET :: "main.jai";

// A jacket.toml wins over a jaifmt.toml in the same directory, as in the native jaifmt.
CONFIG_FILE_NAMES :: string.[JACKET_CONFIG_FILE_NAME, FORMAT_CONFIG_FILE_NAME];

main :: () -> int {
    path := tprint("%/%", WORKSPACE, TARGET);
    source, read_ok := read_entire_file(path, log_errors = false);

    if !read_ok return fail("%: cannot read the file", TARGET);

    config: Format_Config;
    directory := parent_directory(path);

    found := false;

    while true {
        for name: CONFIG_FILE_NAMES {
            toml, has_config := read_entire_file(tprint("%/%", directory, name), log_errors = false);
            if !has_config continue;

            parsed: Format_Config;
            ok: bool;
            error: string;

            if name == JACKET_CONFIG_FILE_NAME {
                parsed, ok, error = parse_jacket_config(toml);
            } else {
                parsed, ok, error = parse_config(toml);
            }

            if !ok return fail("%/%: %", directory, name, error);

            config = parsed;
            found = true;
            break;
        }

        if found break;

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
#import "Extensions/Jai_Format";

WORKSPACE :: "/workspace";

fail :: (format: string, args: ..Any) -> int {
    print("jaifmt: %\n", tprint(format, ..args), to_standard_error = true);
    return 1;
}

parent_directory :: (path: string) -> string {
    for < path {
        if it == #char "/" return ifx it_index == 0 then "/" else string.{ it_index, path.data };
    }

    return WORKSPACE;
}
