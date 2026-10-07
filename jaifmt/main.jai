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

USAGE :: #string END
usage: jaifmt [options] <file or directory>...

Formats Jai source files in place. Directories are searched recursively for *.jai files.

options:
  --check          do not write; list the files that would change and exit with 1
  --stdin          format standard input to standard output
  --config <file>  use this config instead of the nearest jaifmt.toml
  --verbose, -v    report every file, a summary, and lines over max_width
  --color <when>   colour errors: auto (on a terminal, unless NO_COLOR is set), always, never
  --help, -h       show this text

exit status: 0 success, 1 files would change (--check), 2 errors
END

Options :: struct {
    check: bool;
    stdin: bool;
    verbose: bool;
    config_path: string;
    paths: [..] string;
}

Totals :: struct {
    checked: int;
    changed: int;
    ignored: int;
    errors: int;
    long_lines: int;
}

main :: () {
    args := get_command_line_arguments();
    options: Options;
    i := 1;
    while i < args.count {
        arg := args[i];
        i += 1;
        if arg == {
            case "--check"; options.check = true;
            case "--stdin"; options.stdin = true;
            case "--verbose"; options.verbose = true;
            case "-v"; options.verbose = true;
            case "--help"; #through;
            case "-h";
                print("%", USAGE);
                exit(0);
            case "--config";
                if i >= args.count fatal_usage("`--config` needs a file", "for example `--config path/to/jaifmt.toml`");
                options.config_path = args[i];
                i += 1;
            case "--color";
                if i >= args.count fatal_usage("`--color` needs a value", "use auto, always or never");
                set_color(args[i]);
                i += 1;
            case;
                if begins_with(arg, "--color=") {
                    set_color(slice(arg, 8, arg.count - 8));
                } else if begins_with(arg, "-") {
                    fatal_usage(tprint("unknown option `%`", arg), "`jaifmt --help` lists the options");
                } else {
                    array_add(*options.paths, arg);
                }
        }
    }

    explicit_config: *Format_Config;
    if options.config_path.count {
        config, ok, message := load_config(absolute_path(options.config_path));
        if !ok {
            report_error(message);
            exit(2);
        }
        explicit_config = config;
    }

    if options.stdin {
        if options.paths.count fatal_usage("`--stdin` takes no paths", "drop the paths, or drop `--stdin` to format the files in place");
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

color_choice := 0; // 0 auto, 1 always, 2 never

set_color :: (when: string) {
    if when == {
        case "auto"; color_choice = 0;
        case "always"; color_choice = 1;
        case "never"; color_choice = 2;
        case; fatal_usage(tprint("unknown `--color` value `%`", when), "use auto, always or never");
    }
}

color_on :: (code: string) -> string {
    return ifx use_color() then code else "";
}

color_off :: () -> string {
    return ifx use_color() then "\e[0m" else "";
}

// `--color auto` decides as jaic and jailint do (`jaic::render::detect_with`): NO_COLOR turns
// colour off, a FORCE_COLOR or CLICOLOR_FORCE that is not empty or "0" on, and otherwise it is
// on for a terminal that is not "dumb" (on Windows, a console known to understand ANSI codes).
use_color :: () -> bool {
    if color_choice == 1 return true;
    if color_choice == 2 return false;
    if environment_set("NO_COLOR") return false;
    if forced("FORCE_COLOR") || forced("CLICOLOR_FORCE") return true;
    if environment_value("TERM") == "dumb" return false;
    #if OS == .WINDOWS {
        ansi := environment_set("WT_SESSION") || environment_set("ANSICON") || environment_value("ConEmuANSI") == "ON" || environment_set("TERM");
        mode: u32;
        return ansi && Windows.GetConsoleMode(Windows.GetStdHandle(Windows.STD_ERROR_HANDLE), *mode) != 0;
    } else {
        return POSIX.isatty(2) != 0;
    }
}

forced :: (name: string) -> bool {
    value := environment_value(name);
    return value.count > 0 && value != "0";
}

environment_set :: (name: string) -> bool {
    return environment_variable(name) != null;
}

// The variable's value, empty when it is not set.
environment_value :: (name: string) -> string {
    value := environment_variable(name);
    return ifx value then to_string(value) else "";
}

environment_variable :: (name: string) -> *u8 {
    #if OS == .WINDOWS return Windows.getenv(temp_c_string(name));
    else return POSIX.getenv(temp_c_string(name));
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
