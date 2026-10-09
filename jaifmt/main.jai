// jaifmt: a token-based formatter for Jai source. See docs/tools/jaifmt.md.
//
//     jaic build jaifmt/build.jai    (or: jaic build jaifmt/main.jai -O2 -o target/jaifmt)
//     jaifmt [--check] [--config file] [--verbose] paths...
//     jaifmt --stdin [--config file] < in.jai > out.jai

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";

#import "Extensions/Jai_Format";

Args :: #import "Extensions/Args";

Options :: struct {
    paths: [..] string; @positional @conflicts=stdin
    check: bool;
    stdin: bool;
    config: string; @value=FILE
    verbose: bool; @short=v
    color: Args.Color_Mode = .AUTO;
}

HELP :: Args.Help(Options).{
    about = "Formats Jai source files in place. Directories are searched recursively for *.jai files.\n\nExit status: 0 success, 1 files would change (--check), 2 errors.",
    paths = "Files or directories to format",
    check = "Do not write; list the files that would change and exit with 1",
    stdin = "Format standard input to standard output",
    config = "Use this config instead of the nearest jaifmt.toml",
    verbose = "Report every file, a summary, and lines over max_width",
    color = "Colour errors: auto (on a terminal, unless NO_COLOR is set), always, never",
};

Totals :: struct {
    checked: int;
    changed: int;
    ignored: int;
    errors: int;
    long_lines: int;
}

main :: () {
    options := Args.parse(Options, HELP);

    explicit_config: *Format_Config;
    if options.config.count {
        config, ok, message := load_config(absolute_path(options.config));
        if !ok {
            report_error(message);
            exit(2);
        }
        explicit_config = config;
    }

    if options.stdin {
        exit(cast(s32) format_stdin(explicit_config));
    }
    if !options.paths.count fatal_usage("no files to format", "pass files or directories, as in `jaifmt src`, or use `jaifmt --stdin < file.jai`");

    files: [..] string;
    totals: Totals;
    for options.paths {
        path := absolute_path(it);
        is_dir, exists := is_directory(path);
        if !exists {
            report_error(tprint("`%` does not exist", it), "relative paths start from the current directory");
            totals.errors += 1;
            continue;
        }
        if is_dir {
            collect_files(path, explicit_config, *files, *totals);
        } else {
            config := config_for(path, explicit_config, *totals);
            if !config continue;
            if format_is_ignored(config.*, path) {
                totals.ignored += 1;
                continue;
            }
            array_add(*files, path);
        }
    }
    quick_sort(files, compare_strings);

    for path: files {
        format_file(path, explicit_config, options, *totals);
        reset_temporary_storage();
    }

    if options.verbose {
        verb := ifx options.check then "would change" else "reformatted";
        print_error("jaifmt: % files checked, % %, % ignored, % errors", totals.checked, totals.changed, verb, totals.ignored, totals.errors);
        if totals.long_lines print_error(", % lines over max_width", totals.long_lines);
        print_error("\n");
    }
    if totals.errors exit(2);
    if options.check && totals.changed exit(1);
    exit(0);
}

fatal_usage :: (message: string, help := "") {
    report_error(message, help);
    exit(2);
}

// Errors go to stderr as `error: ...`, then an optional `help: ...` line.
report_error :: (message: string, help := "") {
    print_error("%error%: %\n", color_on(ERROR_COLOR), color_off(), message);
    if help.count print_error("%help%: %\n", color_on(HELP_COLOR), color_off(), help);
}

// A formatting error from Jai_Format starts with `line:column: ` when it has a place.
report_format_error :: (path: string, error: string) {
    rest := error;
    place := 0;
    for 1..2 {
        digits := 0;
        while digits < rest.count && rest[digits] >= #char "0" && rest[digits] <= #char "9" digits += 1;
        if !digits || digits >= rest.count || rest[digits] != #char ":" break;
        place += digits + 1;
        rest = slice(rest, digits + 1, rest.count - digits - 1);
    }
    if place && begins_with(rest, " ") {
        print_error("%:% ", shown(path), slice(error, 0, place));
        report_error(slice(rest, 1, rest.count - 1), "the file was left unchanged; jaifmt formats only code whose brackets balance");
        return;
    }
    print_error("%: ", shown(path));
    report_error(error, "the file was left unchanged");
}

ERROR_COLOR :: "\e[1;31m";
HELP_COLOR :: "\e[1;36m";

color_on :: (code: string) -> string {
    return ifx Args.color_enabled(.ERROR) then code else "";
}

color_off :: () -> string {
    return ifx Args.color_enabled(.ERROR) then "\e[0m" else "";
}

print_error :: (format: string, args: ..Any) {
    print(format, ..args, to_standard_error = true);
}

// Paths are kept with forward slashes, which Windows accepts too; `C:/...` counts as absolute.
absolute_path :: (path: string) -> string {
    slashed := replace(path, "\\", "/",, temp);
    if begins_with(slashed, "/") || (slashed.count > 1 && slashed[1] == #char ":") {
        return copy_string(trim_trailing_slashes(slashed));
    }
    cwd := replace(get_working_directory(), "\\", "/",, temp);
    return copy_string(trim_trailing_slashes(tprint("%/%", cwd, slashed)));
}

// `path` relative to the working directory when it lies inside it, for messages.
shown :: (path: string) -> string {
    cwd := tprint("%/", replace(get_working_directory(), "\\", "/",, temp));
    if begins_with(path, cwd) return slice(path, cwd.count, path.count - cwd.count);
    return path;
}

trim_trailing_slashes :: (path: string) -> string {
    result := path;
    while result.count > 1 && result[result.count - 1] == #char "/" result.count -= 1;
    while begins_with(result, "./") result = slice(result, 2, result.count - 2);
    return result;
}

config_for :: (path: string, explicit_config: *Format_Config, totals: *Totals) -> *Format_Config {
    if explicit_config return explicit_config;
    config, ok, message := config_for_directory(parent_directory(path));
    if !ok {
        report_error(message);
        totals.errors += 1;
        return null;
    }
    return config;
}

Walk :: struct {
    files: *[..] string;
    totals: *Totals;
    explicit_config: *Format_Config;
}

collect_files :: (directory: string, explicit_config: *Format_Config, files: *[..] string, totals: *Totals) {
    walk: Walk;
    walk.files = files;
    walk.totals = totals;
    walk.explicit_config = explicit_config;
    visitor :: (info: *File_Visit_Info, walk: *Walk) {
        if begins_with(info.short_name, ".") {
            info.descend_into_directory = false;
            return;
        }
        if !info.is_directory && !ends_with(info.short_name, ".jai") return;
        config := config_for(info.full_name, walk.explicit_config, walk.totals);
        if !config {
            info.descend_into_directory = false;
            return;
        }
        if format_is_ignored(config.*, info.full_name) {
            info.descend_into_directory = false;
            if !info.is_directory walk.totals.ignored += 1;
            return;
        }
        if !info.is_directory array_add(walk.files, copy_string(info.full_name));
    }
    visit_files(directory, true, *walk, visitor, visit_directories = true, follow_directory_symlinks = false);
}

format_file :: (path: string, explicit_config: *Format_Config, options: Options, totals: *Totals) {
    config := config_for(path, explicit_config, totals);
    if !config return;
    source, ok := read_entire_file(path, log_errors = false);
    if !ok {
        report_error(tprint("could not read `%`", shown(path)), "check that the file is readable");
        totals.errors += 1;
        return;
    }
    defer free(source);
    totals.checked += 1;

    output, formatted, error := format_source(source, config.*);
    if !formatted {
        report_format_error(path, error);
        totals.errors += 1;
        return;
    }
    defer free(output);
    long_lines := format_count_long_lines(output, config.max_width);
    totals.long_lines += long_lines;
    if options.verbose && long_lines {
        print_error("jaifmt: %: % lines over % columns\n", shown(path), long_lines, config.max_width);
    }
    if output == source return;

    totals.changed += 1;
    if options.check {
        line := first_difference_line(source, output);
        print("%:%\n", shown(path), line);
        return;
    }
    if !write_entire_file(path, output.data, output.count) {
        report_error(tprint("could not write `%`", shown(path)), "check that the file and its directory are writable");
        totals.errors += 1;
        return;
    }
    if options.verbose print_error("jaifmt: formatted %\n", shown(path));
}

first_difference_line :: (a: string, b: string) -> int {
    line := 1;
    for i: 0..min(a.count, b.count) - 1 {
        if a[i] != b[i] return line;
        if a[i] == #char "\n" line += 1;
    }
    return line;
}

format_stdin :: (explicit_config: *Format_Config) -> int {
    source := read_all_stdin();
    config := explicit_config;
    if !config {
        found, ok, message := config_for_directory(get_working_directory());
        if !ok {
            report_error(message);
            return 2;
        }
        config = found;
    }
    output, formatted, error := format_source(source, config.*);
    if !formatted {
        report_format_error("<stdin>", error);
        return 2;
    }
    write_string(output);
    return 0;
}

// Pipes have no length, so read in chunks until end of input.
read_all_stdin :: () -> string {
    builder: String_Builder;
    buffer: [65536] u8;
    while true {
        n := read_stdin_chunk(buffer.data, buffer.count);
        if n <= 0 break;
        append(*builder, buffer.data, n);
    }
    return builder_to_string(*builder);
}

#if OS == .WINDOWS {
    Windows :: #import "Windows";

    // A pipe at end of input reports ERROR_BROKEN_PIPE, which ends the loop like a 0 read.
    read_stdin_chunk :: (buffer: *u8, size: int) -> int {
        got: u32;
        if !Windows.ReadFile(Windows.GetStdHandle(Windows.STD_INPUT_HANDLE), buffer, cast(u32) size, *got, null) return -1;
        return got;
    }
} else {
    POSIX :: #import "POSIX";

    read_stdin_chunk :: (buffer: *u8, size: int) -> int {
        return POSIX.read(POSIX.STDIN_FILENO, buffer, cast(u64) size);
    }
}

// Finds the nearest jaifmt.toml in `directory` or its parents and loads it; returns defaults
// when there is none. Results are cached per directory.
config_for_directory :: (directory: string) -> *Format_Config, ok: bool, error: string {
    for config_cache if it.directory == directory return it.config, it.ok, it.error;

    entry: Config_Cache_Entry;
    entry.directory = copy_string(directory);
    candidate := tprint("%/%", directory, FORMAT_CONFIG_FILE_NAME);
    if file_exists(candidate) {
        entry.config, entry.ok, entry.error = load_config(candidate);
    } else {
        parent := parent_directory(directory);
        if parent.count && parent != directory {
            entry.config, entry.ok, entry.error = config_for_directory(parent);
        } else {
            entry.config = New(Format_Config);
            entry.ok = true;
        }
    }
    array_add(*config_cache, entry);
    return entry.config, entry.ok, entry.error;
}

load_config :: (path: string) -> *Format_Config, ok: bool, error: string {
    config := New(Format_Config);
    text, read_ok := read_entire_file(path, log_errors = false);
    if !read_ok return config, false, sprint("could not read config `%`", shown(path));
    parsed, ok, message := parse_config(text);
    if !ok return config, false, sprint("in `%`, %", shown(path), message);
    config.* = parsed;
    config.root = copy_string(parent_directory(path));
    return config, true, "";
}

// `a/b/c` -> `a/b`; `/a` -> `/`; `a` -> "".
parent_directory :: (path: string) -> string {
    end := path.count;
    while end > 1 && path[end - 1] == #char "/" end -= 1;
    slash := find_index_from_right(slice(path, 0, end), #char "/");
    if slash < 0 return "";
    if slash == 0 return "/";
    return slice(path, 0, slash);
}

Config_Cache_Entry :: struct {
    directory: string;
    config: *Format_Config;
    ok: bool;
    error: string;
}

config_cache: [..] Config_Cache_Entry;
