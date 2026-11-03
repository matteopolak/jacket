// A source formatter for Jai: indentation, spacing, brace placement and blank lines. It is text
// in, text out, with no file access of its own. [format_source] formats, [parse_config] reads the
// `[format]` section of a `jacket.toml`, and [format_find_config] finds the right one for a file.
//
// ```jai
// text := "main::()  {\nx:=1;\n}\n";
// formatted, ok, error := format_source(text);
// assert(ok);
// assert(formatted == "main :: () {\n    x := 1;\n}\n");
// ```
//
// Formatting never changes the token stream. The result is lexed again and compared with the
// input, and on any difference `format_source` fails instead of returning text.

// Where an opening brace goes.
Brace_Style :: enum u8 {
    SAME_LINE; // `if x\n{` becomes `if x {`, and `}\nelse` becomes `} else`
    PRESERVE;
}

// The formatter's settings, which are the keys of the `[format]` table in `jacket.toml`. The
// defaults are what you get with no config. Lines are never rewrapped; `max_width` only feeds
// [format_count_long_lines].
//
// ```jai
// config: Format_Config;
// config.indent_width = 2;
// formatted, ok := format_source("main :: () {\nx := 1;\n}\n", config);
// assert(ok && formatted == "main :: () {\n  x := 1;\n}\n");
// ```
Format_Config :: struct {
    indent_width := 4;
    case_indent := -1; // `case` lines inside `if x == {`; -1: indent_width
    case_body_indent := -1; // statements under a `case`; -1: indent_width
    max_blank_lines := 2;
    max_width := 100; // only reported (format_count_long_lines); lines are never rewrapped
    brace_style := Brace_Style.SAME_LINE;
    blank_lines_between_items := true; // a blank line around multi-line file-level items and import groups
    blank_line_after_declarations := false; // in code blocks: a blank line after a run of declarations
    blank_lines_around_blocks := false; // in code blocks: a blank line before and after statements with a block body
    blank_line_between_cases := false; // in a switch: a blank line before a `case` that follows a multi-line arm
    blank_line_before_comments := false; // in code blocks: a blank line above a comment that follows code
    blank_line_after_guards := false; // in code blocks: a blank line after a one-line `if`, `for` or `while`
    blank_line_before_return := false; // in code blocks: a blank line above a `return`
    ignore: [..] string; // globs relative to `root`, for format_is_ignored
    unknown_keys: [..] string; // keys the parser did not know and dropped; see format_unknown_setting_note
    root: string; // directory of the config file; set by whoever read it
}

// The config file the command-line tool looks for, upward from each file (its `[format]` section).
JACKET_CONFIG_FILE_NAME :: "jacket.toml";

// Formats Jai source. On success `result` is newly allocated (free it with `free`); on failure
// it is empty and `error` (temporary storage) says why, with a `line:column:` prefix when it
// points at the input: text that does not lex, unbalanced brackets, or the internal token check.
//
// ```jai
// formatted, ok, error := format_source("f::(a:int)->int{return a;}\n");
// assert(ok);
// assert(formatted == "f :: (a: int) -> int {\n    return a;\n}\n");
//
// _, ok, error = format_source("f :: () {");
// assert(!ok && error.count > 0);
// ```
format_source :: (text: string, config: Format_Config = .{}) -> result: string, ok: bool, error: string {
    formatted := format_text(text, config);
    if !formatted.ok return "", false, formatted.error;
    return formatted.output, true, "";
}

// Parses the `[format]` section of jacket.toml text (see the docs for the keys). Other tables are
// skipped; without a `[format]` section the result is the default config. Bad values are errors,
// reported as `line N: ...`; unknown keys are dropped and listed in `unknown_keys`.
//
// ```jai
// config, ok, error := parse_config("[format]\nindent_width = 2\nbrace_style = \"preserve\"\n");
// assert(ok);
// assert(config.indent_width == 2 && config.brace_style == .PRESERVE);
// ```
parse_config :: (text: string) -> config: Format_Config, ok: bool, error: string {
    config: Format_Config;
    ok, error := parse_config_into(text, *config);
    return config, ok, error;
}

// Sets one setting from `key=value`, the value written as in the TOML file (`4`, `true`,
// `"preserve"`): how a caller that already resolved the settings passes them on. Unlike the file
// parser, a key the formatter does not have is an error.
//
// ```jai
// config: Format_Config;
// ok, error := format_apply_setting(*config, "indent_width=8");
// assert(ok && config.indent_width == 8);
// ok, error = format_apply_setting(*config, "colour=red");
// assert(!ok);
// ```
format_apply_setting :: (config: *Format_Config, assignment: string) -> ok: bool, error: string {
    split := find_index_from_left(assignment, #char "=");
    if split < 0 return false, tprint("`%` is not key=value", assignment);

    key := trim(slice(assignment, 0, split));
    value := trim(slice(assignment, split + 1, assignment.count - split - 1));

    known := false;

    for FORMAT_KEYS if it == key known = true;
    if !known return false, tprint("unknown setting `%`", key);

    ok, message := apply_key(config, key, value, 0);
    if !ok return false, without_line_prefix(message);

    return true, "";
} @TempAllocates

// The warning for an entry of `config.unknown_keys` found in the file `file`, with a suggestion
// when a known key is close.
//
// ```jai
// config := parse_config("[format]\nindent_wdth = 2\n");
// assert(config.unknown_keys.count == 1);
// note := format_unknown_setting_note(config.unknown_keys[0], "jacket.toml");
// assert(note.count > 0);
// ```
format_unknown_setting_note :: (key: string, file: string) -> string {
    return unknown_setting_note(key, file);
} @TempAllocates

// Whether `before` and `after` are the same program as far as the compiler can tell: the same
// tokens, comments and the line breaks and adjacency the parser depends on. `format_source`
// already checks its own output with this.
//
// ```jai
// ok, why := format_tokens_equivalent("x:=1;", "x := 1;");
// assert(ok);
// ok, why = format_tokens_equivalent("x := 1;", "x := 2;");
// assert(!ok && why.count > 0);
// ```
format_tokens_equivalent :: (before: string, after: string) -> ok: bool, why: string {
    ok, why := check_equivalent(before, after);
    return ok, why;
}

// Is `path` ignored by `config.ignore`? Globs are relative to `config.root`: `*` and `?` stay
// within a path component, `**` crosses components, and a matched directory covers its contents.
//
// ```jai
// config, ok := parse_config("[format]\nignore = [\"vendor\", \"**/generated_*.jai\"]\n");
// config.root = "/project";
// assert(format_is_ignored(config, "/project/vendor/lib/a.jai"));
// assert(format_is_ignored(config, "/project/src/deep/generated_x.jai"));
// assert(!format_is_ignored(config, "/project/src/main.jai"));
// ```
format_is_ignored :: (config: Format_Config, path: string) -> bool {
    return is_ignored(config, path);
}

// Whether the glob `pattern` matches the path `text`, with the rules of [format_is_ignored].
//
// ```jai
// assert(format_glob_match("src/*.jai", "src/a.jai"));
// assert(!format_glob_match("src/*.jai", "src/deep/a.jai"));
// assert(format_glob_match("**/a.jai", "src/deep/a.jai"));
// ```
format_glob_match :: (pattern: string, text: string) -> bool {
    return glob_path_match(pattern, text);
}

// Reads the file at `path` for `format_find_config`: its text, and whether it could be read.
Format_File_Reader :: #type (path: string) -> text: string, ok: bool;

// The settings for files in `directory`, from the nearest jacket.toml at or above it (up to
// `stop`), with `read` supplying file contents. `file` is the `jacket.toml` that was used, or ""
// for the defaults.
//
// ```jai
// reader :: (path: string) -> string, bool {
//     if path == "/project/jacket.toml" return "[format]\nindent_width = 2\n", true;
//     return "", false;
// }
//
// config, file, ok := format_find_config("/project/src", reader);
// assert(ok && file == "/project/jacket.toml" && config.indent_width == 2);
// ```
format_find_config :: (directory: string, read: Format_File_Reader, stop := "") -> config: Format_Config, file: string, ok: bool, error: string {
    config, file, ok, error := find_config(directory, read, stop);
    return config, file, ok, error;
} @TempAllocates(1,3)

// `a/b/c` -> `a/b`; `/a` -> `/`; `a` -> "".
//
// ```jai
// assert(format_parent_directory("a/b/c") == "a/b");
// assert(format_parent_directory("/a") == "/");
// ```
format_parent_directory :: (path: string) -> string {
    return parent_directory_of(path);
}

// Lines in `text` longer than `max_width` code points.
//
// ```jai
// assert(format_count_long_lines("short\na much longer line\n", 10) == 1);
// ```
format_count_long_lines :: (text: string, max_width: int) -> int {
    return count_long_lines(text, max_width);
}

#scope_module

#import "Basic";
#import "String";

#load "lexer.jai";
#load "config.jai";
#load "format.jai";
#load "project.jai";
