// jaifmt: a token-based formatter for Jai source. See docs/tools/jaifmt.md.
//
//     jaic build tools/jaifmt/main.jai -O2 -o jaifmt
//     jaifmt [--check] [--config file] [--verbose] paths...
//     jaifmt --stdin [--config file] < in.jai > out.jai

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "POSIX";
#import "Sort";

#load "lexer.jai";
#load "config.jai";
#load "format.jai";

USAGE :: #string END
usage: jaifmt [options] <file or directory>...

Formats Jai source files in place. Directories are searched recursively for *.jai files.

options:
  --check          do not write; list the files that would change and exit with 1
  --stdin          format standard input to standard output
  --config <file>  use this config instead of the nearest jaifmt.toml
  --verbose, -v    report every file, a summary, and lines over max_width
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
            case "--check";        options.check = true;
            case "--stdin";        options.stdin = true;
            case "--verbose";      options.verbose = true;
            case "-v";             options.verbose = true;
            case "--help";         #through;
            case "-h";
                print("%", USAGE);
                exit(0);
            case "--config";
                if i >= args.count fatal_usage("--config needs a file");
                options.config_path = args[i];
                i += 1;
            case;
                if begins_with(arg, "-") fatal_usage(tprint("unknown option %", arg));
                array_add(*options.paths, arg);
        }
    }

    explicit_config: *Config;
    if options.config_path.count {
        config, ok, message := load_config(absolute_path(options.config_path));
        if !ok {
            print_error("jaifmt: %\n", message);
            exit(2);
        }
        explicit_config = config;
    }

    if options.stdin {
        if options.paths.count fatal_usage("--stdin takes no paths");
        exit(cast(s32) format_stdin(explicit_config));
    }
    if !options.paths.count fatal_usage("no paths given");

    files: [..] string;
    totals: Totals;
    for options.paths {
        path := absolute_path(it);
        is_dir, exists := is_directory(path);
        if !exists {
            print_error("jaifmt: %: no such file or directory\n", it);
            totals.errors += 1;
            continue;
        }
        if is_dir {
            collect_files(path, explicit_config, *files, *totals);
        } else {
            config := config_for(path, explicit_config, *totals);
            if !config continue;
            if is_ignored(config, path) {
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

fatal_usage :: (message: string) {
    print_error("jaifmt: %\n\n%", message, USAGE);
    exit(2);
}

print_error :: (format: string, args: .. Any) {
    text := tprint(format, ..args);
    write(STDERR_FILENO, text.data, cast(u64) text.count);
}

absolute_path :: (path: string) -> string {
    if begins_with(path, "/") return copy_string(trim_trailing_slashes(path));
    cwd := get_working_directory();
    return copy_string(trim_trailing_slashes(tprint("%/%", cwd, path)));
}

trim_trailing_slashes :: (path: string) -> string {
    result := path;
    while result.count > 1 && result[result.count - 1] == #char "/"  result.count -= 1;
    while begins_with(result, "./") result = slice(result, 2, result.count - 2);
    return result;
}

config_for :: (path: string, explicit_config: *Config, totals: *Totals) -> *Config {
    if explicit_config return explicit_config;
    config, ok, message := config_for_directory(parent_directory(path));
    if !ok {
        print_error("jaifmt: %\n", message);
        totals.errors += 1;
        return null;
    }
    return config;
}

Walk :: struct {
    files: *[..] string;
    totals: *Totals;
    explicit_config: *Config;
}

collect_files :: (directory: string, explicit_config: *Config, files: *[..] string, totals: *Totals) {
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
        if is_ignored(config, info.full_name) {
            info.descend_into_directory = false;
            if !info.is_directory walk.totals.ignored += 1;
            return;
        }
        if !info.is_directory array_add(walk.files, copy_string(info.full_name));
    }
    visit_files(directory, true, *walk, visitor, visit_directories = true, follow_directory_symlinks = false);
}

format_file :: (path: string, explicit_config: *Config, options: Options, totals: *Totals) {
    config := config_for(path, explicit_config, totals);
    if !config return;
    source, ok := read_entire_file(path, log_errors = false);
    if !ok {
        print_error("jaifmt: %: cannot read\n", path);
        totals.errors += 1;
        return;
    }
    defer free(source);
    totals.checked += 1;

    result := format_source(source, config.*);
    if !result.ok {
        print_error("jaifmt: %:%\n", path, result.error);
        totals.errors += 1;
        return;
    }
    defer free(result.output);
    totals.long_lines += result.long_lines;
    if options.verbose && result.long_lines {
        print_error("jaifmt: %: % lines over % columns\n", path, result.long_lines, config.max_width);
    }
    if result.output == source return;

    totals.changed += 1;
    if options.check {
        line := first_difference_line(source, result.output);
        print("%:%\n", path, line);
        return;
    }
    if !write_entire_file(path, result.output.data, result.output.count) {
        print_error("jaifmt: %: cannot write\n", path);
        totals.errors += 1;
        return;
    }
    if options.verbose print_error("jaifmt: formatted %\n", path);
}

first_difference_line :: (a: string, b: string) -> int {
    line := 1;
    for i: 0..min(a.count, b.count) - 1 {
        if a[i] != b[i] return line;
        if a[i] == #char "\n" line += 1;
    }
    return line;
}

format_stdin :: (explicit_config: *Config) -> int {
    source := read_all_stdin();
    config := explicit_config;
    if !config {
        found, ok, message := config_for_directory(get_working_directory());
        if !ok {
            print_error("jaifmt: %\n", message);
            return 2;
        }
        config = found;
    }
    result := format_source(source, config.*);
    if !result.ok {
        print_error("jaifmt: <stdin>:%\n", result.error);
        return 2;
    }
    write(STDOUT_FILENO, result.output.data, cast(u64) result.output.count);
    return 0;
}

// Pipes have no length, so read in chunks until end of input.
read_all_stdin :: () -> string {
    builder: String_Builder;
    buffer: [65536] u8;
    while true {
        n := read(STDIN_FILENO, buffer.data, buffer.count);
        if n <= 0 break;
        append(*builder, buffer.data, n);
    }
    return builder_to_string(*builder);
}
