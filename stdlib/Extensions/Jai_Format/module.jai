// Jai_Format: a source formatter for Jai (indentation, spacing, brace placement, blank lines).
// Pure text in, text out: no file access, so it runs the same natively, under `jaic run` and in
// the browser playground. `jaifmt/` is the command-line front end. See docs/tools/jaifmt.md.
//
//     config, ok, error := parse_config(read_entire_file("jaifmt.toml"));
//     formatted, ok, error := format_source(text, config);
//
// Formatting never changes the token stream: the result is lexed again and compared with the
// input, and on any difference `format_source` fails instead of returning text.

Brace_Style :: enum u8 {
    SAME_LINE; // `if x\n{` becomes `if x {`, and `}\nelse` becomes `} else`
    PRESERVE;
}

Format_Config :: struct {
    indent_width := 4;
    case_indent := -1; // `case` lines inside `if x == {`; -1: indent_width
    case_body_indent := -1; // statements under a `case`; -1: indent_width
    max_blank_lines := 2;
    max_width := 100; // only reported (format_count_long_lines); lines are never rewrapped
    brace_style := Brace_Style.SAME_LINE;
    blank_lines_between_items := true; // a blank line around multi-line file-level items and import groups
    ignore: [..] string; // globs relative to `root`, for format_is_ignored
    root: string; // directory of the config file; set by whoever read it
}

// The config file name the command-line tool looks for, upward from each file.
FORMAT_CONFIG_FILE_NAME :: "jaifmt.toml";

// Formats Jai source. On success `result` is newly allocated (free it with `free`); on failure
// it is empty and `error` (temporary storage) says why, with a `line:column:` prefix when it
// points at the input: text that does not lex, unbalanced brackets, or the internal token check.
format_source :: (text: string, config: Format_Config = .{}) -> result: string, ok: bool, error: string {
    formatted := format_text(text, config);
    if !formatted.ok return "", false, formatted.error;
    return formatted.output, true, "";
}

// Parses jaifmt.toml text (`key = value` lines, `#` comments; see the docs for the keys).
// Unknown keys and bad values are errors, reported as `line N: ...`.
parse_config :: (text: string) -> config: Format_Config, ok: bool, error: string {
    config: Format_Config;
    ok, error := parse_config_into(text, *config);
    return config, ok, error;
}

// Whether `before` and `after` are the same program as far as the compiler can tell: the same
// tokens, comments and the line breaks and adjacency the parser depends on. `format_source`
// already checks its own output with this.
format_tokens_equivalent :: (before: string, after: string) -> ok: bool, why: string {
    ok, why := check_equivalent(before, after);
    return ok, why;
}

// Is `path` ignored by `config.ignore`? Globs are relative to `config.root`: `*` and `?` stay
// within a path component, `**` crosses components, and a matched directory covers its contents.
format_is_ignored :: (config: Format_Config, path: string) -> bool {
    return is_ignored(config, path);
}

format_glob_match :: (pattern: string, text: string) -> bool {
    return glob_match(pattern, text);
}

// Lines in `text` longer than `max_width` code points.
format_count_long_lines :: (text: string, max_width: int) -> int {
    return count_long_lines(text, max_width);
}

#scope_module

#import "Basic";
#import "String";

#load "lexer.jai";
#load "config.jai";
#load "format.jai";
