// jaifmt.toml and jacket.toml `[format]` parsing, and ignore globs. No file access: the caller reads the file.

#scope_module

resolved_case_indent :: (config: Format_Config) -> int {
    if config.case_indent < 0 return config.indent_width;
    return config.case_indent;
}

resolved_case_body_indent :: (config: Format_Config) -> int {
    if config.case_body_indent < 0 return config.indent_width;
    return config.case_body_indent;
}

// Parses `text` into `config`; on failure returns false and a message naming the line. Keys the
// format settings do not have are dropped into `config.unknown_keys` (as `key`, or `format.key`
// for a sub-table), not errors. With
// `jacket`, `text` is a jacket.toml: only the settings under `format` count and everything else
// is skipped; otherwise `text` is a jaifmt.toml, whose keys sit at the top.
//
// The TOML read is the part a config uses: `key = value` pairs, `[table]` headers, bare and quoted
// keys, dotted keys (`format.indent_width = 2`), inline tables (`format = { indent_width = 2 }`),
// basic and literal strings, and arrays or inline tables that continue over several lines.
parse_config_into :: (text: string, config: *Format_Config, jacket := false) -> bool, string {
    line_number := 0;
    rest := text;

    // The keys of the `[table]` header in force.
    table: [..] string;

    // A value may continue over several lines: `statement` collects the lines from the first one
    // until its brackets close, and `depth` counts the brackets still open.
    statement: string;
    statement_line := 0;
    depth := 0;

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

        if depth > 0 {
            statement = tprint("% %", statement, line);
            depth += bracket_delta(line);
        } else {
            if !line.count continue;

            statement = line;
            statement_line = line_number;
            depth = bracket_delta(line);
        }

        if depth > 0 continue;

        depth = 0;

        if statement[0] == #char "[" {
            // Only a jacket.toml has tables.
            if !jacket return false, tprint("line %: expected `key = value`", statement_line);

            header, ok := parse_table_header(statement);
            if !ok return false, tprint("line %: malformed table header", statement_line);

            table = header;

            if table[0] == "format" && table.count > 1 array_add(*config.unknown_keys, dotted(table, 0));
        } else {
            ok, message := apply_pair(config, table, statement, statement_line, jacket);
            if !ok return false, message;
        }
    }

    if depth > 0 return false, tprint("line %: this array or table is never closed", statement_line);

    return true, "";
}

// The keys of a `[a.b]` (or `[[a.b]]`) header line.
parse_table_header :: (header: string) -> [..] string, bool {
    none: [..] string;
    open := 1;

    if begins_with(header, "[[") open = 2;

    if header.count < 2 * open + 1 return none, false;

    for 1..open if header[header.count - it] != #char "]" return none, false;

    path, ok, after := parse_key_path(slice(header, open, header.count - 2 * open));
    if !ok || trim(after).count return none, false;

    return path, true;
}

// One `key = value` statement (its arrays and inline tables are closed) found under the `[table]`
// header whose keys are `table`.
apply_pair :: (config: *Format_Config, table: [] string, statement: string, line_number: int, jacket: bool) -> bool, string {
    in_format := !jacket || (table.count && table[0] == "format");

    path, ok, after := parse_key_path(statement);
    after = trim(after);

    if !ok || !after.count || after[0] != #char "=" {
        if in_format return false, tprint("line %: expected `key = value`", line_number);
        return true, "";
    }

    value := trim(slice(after, 1, after.count - 1));

    keys: [..] string;

    for table array_add(*keys, it);
    for path array_add(*keys, it);

    // In a jacket.toml the format settings are the keys under `format`, however they are written:
    // in a `[format]` table, as `format.name = ...`, or in `format = { ... }`.
    first := 0;

    if jacket {
        if keys[0] != "format" return true, "";

        // A `[format.sub]` table was reported when its header was read.
        if table.count > 1 return true, "";

        first = 1;
    }

    if keys.count == first return apply_inline_table(config, value, line_number);

    return apply_setting(config, keys, first, value, line_number);
}

// The settings in `{ key = value, ... }`, the value of `format` itself.
apply_inline_table :: (config: *Format_Config, value: string, line_number: int) -> bool, string {
    if value.count < 2 || value[0] != #char "{" || value[value.count - 1] != #char "}" {
        return false, tprint("line %: format must be a table", line_number);
    }

    items := slice(value, 1, value.count - 2);

    while true {
        items = trim(items);
        if !items.count break;

        item, after := split_first_item(items);

        items = after;

        path, ok, rest := parse_key_path(item);
        rest = trim(rest);

        if !ok || !rest.count || rest[0] != #char "=" return false, tprint("line %: expected `key = value`", line_number);

        applied, message := apply_setting(config, path, 0, trim(slice(rest, 1, rest.count - 1)), line_number);
        if !applied return false, message;
    }

    return true, "";
}

// Applies `keys` from `first` on as the path of one format setting: a single name is a setting, a
// dotted path names something the formatter does not have.
apply_setting :: (config: *Format_Config, keys: [] string, first: int, value: string, line_number: int) -> bool, string {
    if keys.count - first == 1 return apply_key(config, keys[first], value, line_number);

    array_add(*config.unknown_keys, dotted(keys, first));

    return true, "";
}

// `keys` from `first` on, joined by dots.
dotted :: (keys: [] string, first: int) -> string {
    out: String_Builder;

    for first..keys.count - 1 {
        if it > first append(*out, ".");
        append(*out, keys[it]);
    }

    return builder_to_string(*out);
}

// The text before the first comma that is outside strings and brackets, and the text after it.
split_first_item :: (text: string) -> item: string, rest: string {
    scan: Quote_Scan;
    depth := 0;

    for text {
        if in_string(*scan, it) continue;

        if it == #char "[" || it == #char "{" depth += 1;
        if it == #char "]" || it == #char "}" depth -= 1;

        if it == #char "," && depth == 0 return slice(text, 0, it_index), slice(text, it_index + 1, text.count - it_index - 1);
    }

    return text, "";
}

// The dotted key `a."b c".d` at the start of `text`: its parts, and the text after it.
parse_key_path :: (text: string) -> [..] string, bool, string {
    keys: [..] string;
    rest := trim(text);

    while true {
        rest = trim(rest);
        if !rest.count return keys, false, rest;

        if rest[0] == #char "\"" || rest[0] == #char "'" {
            key, ok, after := parse_quoted_prefix(rest);
            if !ok return keys, false, rest;

            array_add(*keys, key);
            rest = after;
        } else {
            length := 0;
            while length < rest.count && is_bare_key_character(rest[length]) length += 1;
            if !length return keys, false, rest;

            array_add(*keys, copy_string(slice(rest, 0, length)));
            rest = slice(rest, length, rest.count - length);
        }

        rest = trim(rest);

        if !rest.count || rest[0] != #char "." return keys, true, rest;

        rest = slice(rest, 1, rest.count - 1);
    }

    return keys, false, rest;
}

is_bare_key_character :: (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || (c >= #char "0" && c <= #char "9") || c == #char "_" || c == #char "-";
}

// Sets one `key = value` pair; on failure returns false and a message naming the line.
apply_key :: (config: *Format_Config, key: string, value: string, line_number: int) -> bool, string {
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
        case "blank_line_after_declarations";
            ok := parse_flag(value, *config.blank_line_after_declarations);
            if !ok return false, tprint("line %: blank_line_after_declarations must be true or false", line_number);
        case "blank_lines_around_blocks";
            ok := parse_flag(value, *config.blank_lines_around_blocks);
            if !ok return false, tprint("line %: blank_lines_around_blocks must be true or false", line_number);
        case "blank_line_between_cases";
            ok := parse_flag(value, *config.blank_line_between_cases);
            if !ok return false, tprint("line %: blank_line_between_cases must be true or false", line_number);
        case "blank_line_before_comments";
            ok := parse_flag(value, *config.blank_line_before_comments);
            if !ok return false, tprint("line %: blank_line_before_comments must be true or false", line_number);
        case "blank_line_after_guards";
            ok := parse_flag(value, *config.blank_line_after_guards);
            if !ok return false, tprint("line %: blank_line_after_guards must be true or false", line_number);
        case "blank_line_before_return";
            ok := parse_flag(value, *config.blank_line_before_return);
            if !ok return false, tprint("line %: blank_line_before_return must be true or false", line_number);
        case "ignore";
            if !value.count || value[0] != #char "[" || value[value.count - 1] != #char "]" {
                return false, tprint("line %: ignore must be an array of quoted globs", line_number);
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
            array_add(*config.unknown_keys, copy_string(key));
    }

    return true, "";
}

FORMAT_KEYS :: string.["indent_width", "case_indent", "case_body_indent", "max_blank_lines", "max_width", "brace_style", "blank_lines_between_items", "blank_line_after_declarations", "blank_lines_around_blocks", "blank_line_between_cases", "blank_line_before_comments", "blank_line_after_guards", "blank_line_before_return", "ignore"];

// The warning for an unknown `[format]` setting `key` found in the config file `file`: it is
// ignored, and the nearest known key is named when there is one within a third of its length.
unknown_setting_note :: (key: string, file: string) -> string {
    path := key;
    if !begins_with(key, "format.") path = tprint("format.%", key);

    note := tprint("unknown setting `%` in % (ignored)", path, file);

    limit := max(1, key.count / 3);
    best := -1;
    best_distance := limit + 1;

    for FORMAT_KEYS {
        distance := edit_distance(key, it);

        if distance < best_distance {
            best = it_index;
            best_distance = distance;
        }
    }

    if best >= 0 return tprint("%; did you mean `format.%`?", note, FORMAT_KEYS[best]);

    return note;
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
        if glob_path_match(glob, relative) return true;
    }

    return false;
}

// `glob` against a relative path: a trailing `/` on the pattern is dropped, an empty pattern
// matches nothing, and a pattern that matches a directory prefix (`tests/corpus` for
// `tests/corpus/a.jai`) matches the path too.
glob_path_match :: (glob: string, relative: string) -> bool {
    pattern := glob;
    while pattern.count && pattern[pattern.count - 1] == #char "/" pattern.count -= 1;

    if !pattern.count return false;
    if glob_match(pattern, relative) return true;

    for *relative {
        if it.* == #char "/" && glob_match(pattern, slice(relative, 0, it_index)) return true;
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

// Insertions, deletions and substitutions turning `a` into `b`; names past 63 bytes count as far.
edit_distance :: (a: string, b: string) -> int {
    if a.count > 63 || b.count > 63 return 1000;

    previous: [64] int;
    current: [64] int;

    for 0..b.count previous[it] = it;

    for i: 1..a.count {
        current[0] = i;

        for j: 1..b.count {
            cost := 1;
            if a[i - 1] == b[j - 1] cost = 0;

            current[j] = min(previous[j] + 1, min(current[j - 1] + 1, previous[j - 1] + cost));
        }

        for 0..b.count previous[it] = current[it];
    }

    return previous[b.count];
}

// Where a scan along a line of TOML is: inside a basic ("...", with escapes) or a literal ('...')
// string, or not.
Quote_Scan :: struct {
    quote: u8;
    escaped: bool;
}

// Feeds the next character to `scan`; true when it belongs to a string, quotes included.
in_string :: (scan: *Quote_Scan, c: u8) -> bool {
    if scan.quote {
        if scan.escaped {
            scan.escaped = false;
        } else if c == #char "\\" && scan.quote == #char "\"" {
            scan.escaped = true;
        } else if c == scan.quote {
            scan.quote = 0;
        }

        return true;
    }

    if c == #char "\"" || c == #char "'" {
        scan.quote = c;
        return true;
    }

    return false;
}

strip_comment :: (line: string) -> string {
    scan: Quote_Scan;

    for *line {
        if in_string(*scan, it.*) continue;
        if it.* == #char "#" return slice(line, 0, it_index);
    }

    return line;
}

// Brackets and braces opened minus closed in `text`, not counting those inside strings.
bracket_delta :: (text: string) -> int {
    scan: Quote_Scan;
    delta := 0;

    for text {
        if in_string(*scan, it) continue;
        if it == #char "[" || it == #char "{" delta += 1;
        if it == #char "]" || it == #char "}" delta -= 1;
    }

    return delta;
}

parse_count :: (text: string, result: *int, low: int, high: int) -> bool {
    if !text.count return false;

    value := 0;

    for text {
        c := it;
        if c < #char "0" || c > #char "9" return false;
        value = value * 10 + (c - #char "0");
        if value > high return false;
    }

    if value < low return false;

    result.* = value;

    return true;
}

parse_flag :: (text: string, result: *bool) -> bool {
    if text == "true" {
        result.* = true;
        return true;
    }

    if text == "false" {
        result.* = false;
        return true;
    }

    return false;
}

parse_quoted :: (text: string) -> string, bool {
    value, ok, rest := parse_quoted_prefix(text);
    if !ok || trim(rest).count return "", false;
    return value, true;
}

// The string at the start of `text`: basic ("..." with the escapes \" \\ \n \t \r) or literal
// ('...', verbatim). `value` is newly allocated.
parse_quoted_prefix :: (text: string) -> value: string, ok: bool, rest: string {
    if text.count < 2 return "", false, text;

    if text[0] == #char "'" {
        close := find_index_from_left(slice(text, 1, text.count - 1), #char "'");
        if close < 0 return "", false, text;

        return copy_string(slice(text, 1, close)), true, slice(text, close + 2, text.count - close - 2);
    }

    if text[0] != #char "\"" return "", false, text;

    out: String_Builder;
    i := 1;

    while i < text.count {
        c := text[i];

        if c == #char "\"" return builder_to_string(*out), true, slice(text, i + 1, text.count - i - 1);

        if c == #char "\\" {
            i += 1;
            if i >= text.count break;

            if text[i] == #char "n" {
                append(*out, "\n");
            } else if text[i] == #char "t" {
                append(*out, "\t");
            } else if text[i] == #char "r" {
                append(*out, "\r");
            } else if text[i] == #char "\"" || text[i] == #char "\\" {
                append(*out, slice(text, i, 1));
            } else {
                return "", false, text;
            }
        } else {
            append(*out, slice(text, i, 1));
        }

        i += 1;
    }

    return "", false, text;
}
