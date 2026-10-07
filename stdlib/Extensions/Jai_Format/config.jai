// jaifmt.toml parsing and ignore globs. No file access: the caller reads the file.

#scope_module

resolved_case_indent :: (config: Format_Config) -> int {
    if config.case_indent < 0 return config.indent_width;
    return config.case_indent;
}

resolved_case_body_indent :: (config: Format_Config) -> int {
    if config.case_body_indent < 0 return config.indent_width;
    return config.case_body_indent;
}

// Parses `text` into `config`; on failure returns false and a message naming the line.
parse_config_into :: (text: string, config: *Format_Config) -> bool, string {
    line_number := 0;
    rest := text;
    while rest.count {
        line: string;
        newline := find_index_from_left(rest, #char "\n");
        if newline < 0 {
            line = rest;
            rest = "";
        } else {
            line = slice(rest, 0, newline);
            rest = slice(rest, newline + 1, rest.count - newline - 1);
        }
        line_number += 1;
        line = trim(strip_comment(line));
        if !line.count continue;

        equals := find_index_from_left(line, #char "=");
        if equals < 0 return false, tprint("line %: expected `key = value`", line_number);
        key := trim(slice(line, 0, equals));
        value := trim(slice(line, equals + 1, line.count - equals - 1));

        if key == {
            case "indent_width";
                ok := parse_count(value, *config.indent_width, 1, 16);
                if !ok return false, tprint("line %: indent_width must be an integer from 1 to 16", line_number);
            case "case_indent";
                ok := parse_count(value, *config.case_indent, 0, 16);
                if !ok return false, tprint("line %: case_indent must be an integer from 0 to 16", line_number);
            case "case_body_indent";
                ok := parse_count(value, *config.case_body_indent, 0, 16);
                if !ok return false, tprint("line %: case_body_indent must be an integer from 0 to 16", line_number);
            case "max_blank_lines";
                ok := parse_count(value, *config.max_blank_lines, 0, 100);
                if !ok return false, tprint("line %: max_blank_lines must be an integer from 0 to 100", line_number);
            case "max_width";
                ok := parse_count(value, *config.max_width, 10, 10000);
                if !ok return false, tprint("line %: max_width must be an integer from 10 to 10000", line_number);
            case "brace_style";
                style, ok := parse_quoted(value);
                if !ok return false, tprint("line %: brace_style must be a quoted string", line_number);
                if style == "same_line" {
                    config.brace_style = .SAME_LINE;
                } else if style == "preserve" {
                    config.brace_style = .PRESERVE;
                } else {
                    return false, tprint("line %: brace_style must be \"same_line\" or \"preserve\"", line_number);
                }
            case "blank_lines_between_items";
                if value == "true" {
                    config.blank_lines_between_items = true;
                } else if value == "false" {
                    config.blank_lines_between_items = false;
                } else {
                    return false, tprint("line %: blank_lines_between_items must be true or false", line_number);
                }
            case "ignore";
                if !value.count || value[0] != #char "[" || value[value.count - 1] != #char "]" {
                    return false, tprint("line %: ignore must be an array of quoted globs on one line", line_number);
                }
                items := slice(value, 1, value.count - 2);
                while true {
                    items = trim(items);
                    if !items.count break;
                    glob, ok, after := parse_quoted_prefix(items);
                    if !ok return false, tprint("line %: ignore entries must be quoted strings", line_number);
                    array_add(*config.ignore, glob);
                    items = trim(after);
                    if items.count && items[0] == #char "," items = slice(items, 1, items.count - 1);
                }
            case;
                return false, tprint("line %: unknown key `%`", line_number, key);
        }
    }
    return true, "";
}

// Is `path` (absolute) ignored by `config`? Globs are relative to the config's directory:
// `*` matches within one path component, `**` across components, `?` one character.
// A glob that matches a directory also ignores everything under it.
is_ignored :: (config: Format_Config, path: string) -> bool {
    if !config.ignore.count || !config.root.count return false;
    root := config.root;
    if !begins_with(path, root) return false;
    relative := slice(path, root.count, path.count - root.count);
    while relative.count && relative[0] == #char "/" relative = slice(relative, 1, relative.count - 1);
    for glob: config.ignore {
        if glob_match(glob, relative) return true;
        // Directory prefixes: `tests/corpus` ignores `tests/corpus/a.jai`.
        for i: 0..relative.count - 1 {
            if relative[i] == #char "/" && glob_match(glob, slice(relative, 0, i)) return true;
        }
    }
    return false;
}

glob_match :: (pattern: string, text: string) -> bool {
    if !pattern.count return !text.count;
    if begins_with(pattern, "**") {
        rest := slice(pattern, 2, pattern.count - 2);
        if rest.count && rest[0] == #char "/" {
            // `**/x` also matches `x` at the top.
            if glob_match(slice(rest, 1, rest.count - 1), text) return true;
        }
        for i: 0..text.count {
            if glob_match(rest, slice(text, i, text.count - i)) return true;
        }
        return false;
    }
    c := pattern[0];
    if c == #char "*" {
        rest := slice(pattern, 1, pattern.count - 1);
        for i: 0..text.count {
            if glob_match(rest, slice(text, i, text.count - i)) return true;
            if i < text.count && text[i] == #char "/" break;
        }
        return false;
    }
    if !text.count return false;
    if c == #char "?" {
        if text[0] == #char "/" return false;
    } else if c != text[0] {
        return false;
    }
    return glob_match(slice(pattern, 1, pattern.count - 1), slice(text, 1, text.count - 1));
}

#scope_file

strip_comment :: (line: string) -> string {
    in_string := false;
    for i: 0..line.count - 1 {
        c := line[i];
        if c == #char "\"" in_string = !in_string;
        if c == #char "#" && !in_string return slice(line, 0, i);
    }
    return line;
}

parse_count :: (text: string, result: *int, low: int, high: int) -> bool {
    if !text.count return false;
    value := 0;
    for i: 0..text.count - 1 {
        c := text[i];
        if c < #char "0" || c > #char "9" return false;
        value = value * 10 + (c - #char "0");
        if value > high return false;
    }
    if value < low return false;
    result.* = value;
    return true;
}

parse_quoted :: (text: string) -> string, bool {
    value, ok, rest := parse_quoted_prefix(text);
    if !ok || trim(rest).count return "", false;
    return value, true;
}

parse_quoted_prefix :: (text: string) -> value: string, ok: bool, rest: string {
    if text.count < 2 || text[0] != #char "\"" return "", false, text;
    close := find_index_from_left(slice(text, 1, text.count - 1), #char "\"");
    if close < 0 return "", false, text;
    value := copy_string(slice(text, 1, close));
    return value, true, slice(text, close + 2, text.count - close - 2);
}
