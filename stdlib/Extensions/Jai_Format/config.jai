// jacket.toml schema adaptation and ignore globs. The shared TOML module validates whole files.
#scope_module

resolved_case_indent :: (config: Format_Config) -> int {
    if config.case_indent < 0 return config.indent_width;
    return config.case_indent;
}

resolved_case_body_indent :: (config: Format_Config) -> int {
    if config.case_body_indent < 0 return config.indent_width;
    return config.case_body_indent;
}

// Validate the whole file once. Ordered source statements preserve header warnings and raw
// override lexemes; decoded values handle the formatter schema independently of TOML syntax.
parse_config_into :: (text: string, config: *Format_Config) -> bool, string {
    doc, error := Toml.parse_toml(text, .{ record_statements = true });
    if error.kind != .NONE return false, tprint("line %: %", error.line, error.message);

    defer Toml.free_toml(*doc);
    format := Toml.toml_get(doc.root, "format");
    docs := Toml.toml_get(Toml.toml_get(doc.root, "doc"), "format");

    if format && format.kind != .TABLE return false, tprint("line %: format must be a table", config_line(text, format.span.start));
    if docs && docs.kind != .TABLE return false, tprint("line %: doc.format must be a table", config_line(text, docs.span.start));

    // The path helper writes every returned element; reuse storage without clearing an
    // unused tail for every source event. Persisted settings copy their strings below.
    buffer: [128] string;
    index := 0;

    while index < doc.statements.count {
        statement := doc.statements[index];
        path, path_error := Toml.toml_statement_path(*doc, statement, buffer);

        if path_error.kind != .NONE return false, tprint("line %: %", statement.line, path_error.message);

        is_doc := path.count >= 2 && path[0] == "doc" && path[1] == "format";
        is_format := path.count >= 1 && path[0] == "format";

        if !is_doc && !is_format {
            index += 1;
            continue;
        }

        first := ifx is_doc then 2 else 1;

        if statement.kind != .ASSIGNMENT {
            if path.count > first array_add(*config.unknown_keys, config_dotted_path(path, 0));
            index += 1;
            continue;
        }

        if statement.header_depth > first {
            index += 1;
            continue;
        }

        if path.count == first {
            // `format={...}` / `doc.format={...}`: the child assignments carry their own spans.
            index += 1;
            continue;
        }

        if path.count > first + 1 {
            array_add(*config.unknown_keys, config_dotted_path(path, ifx is_doc then 0 else first));
        } else {
            key := path[first];
            if is_doc && !is_doc_format_key(key) {
                array_add(*config.unknown_keys, copy_string(tprint("doc.format.%", key)));
            } else if is_doc {
                scratch: Format_Config;
                ok, message := apply_toml_key(*scratch, key, statement.value, statement.line);

                if !ok return false, message;

                span := statement.value.span;
                raw := slice(doc.source, span.start, span.end - span.start);

                array_add(*config.doc_format, Doc_Setting.{ copy_string(key), copy_string(raw) });
            } else {
                ok, message := apply_toml_key(config, key, statement.value, statement.line);
                if !ok return false, message;
            }
        }

        // A setting's nested value is consumed as a whole, including ignored unknown settings.
        // It must not also produce warnings or overrides for assignments inside its inline tables.
        index += 1;
        while index < doc.statements.count && doc.statements[index].span.start < statement.span.end index += 1;
    }

    return true, "";
}

config_line :: (text: string, offset: int) -> int {
    line := 1;
    for 0..offset - 1 if text[it] == 10 line += 1;
    return line;
}

config_dotted_path :: (keys: [] string, first: int) -> string {
    out: String_Builder;

    defer free_buffers(*out);

    for first..keys.count - 1 {
        if it > first append(*out, ".");
        append(*out, keys[it]);
    }

    return builder_to_string(*out);
}

// The schema consumes decoded types rather than reparsing raw TOML syntax. This accepts all
// valid integer/string spellings while preserving the formatter's range/type diagnostics.
apply_toml_key :: (config: *Format_Config, key: string, value: *Toml.Toml_Value, line_number: int) -> bool, string {
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
            style := value.text;

            if value.kind != .STRING return false, tprint("line %: brace_style must be a quoted string", line_number);

            if style == "same_line" {
                config.brace_style = .SAME_LINE;
            } else if style == "preserve" {
                config.brace_style = .PRESERVE;
            } else {
                return false, tprint("line %: brace_style must be \"same_line\" or \"preserve\"", line_number);
            }
        case "blank_lines_between_items";
            if value.kind == .BOOL && value.boolean {
                config.blank_lines_between_items = true;
            } else if value.kind == .BOOL && !value.boolean {
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
        case "format_doc_examples";
            ok := parse_flag(value, *config.format_doc_examples);
            if !ok return false, tprint("line %: format_doc_examples must be true or false", line_number);
        case "ignore";
            if value.kind != .ARRAY return false, tprint("line %: ignore must be an array of quoted globs", line_number);
            for value.items if it.kind != .STRING return false, tprint("line %: ignore entries must be quoted strings", line_number);
            for value.items array_add(*config.ignore, copy_string(it.text));
        case;
            array_add(*config.unknown_keys, copy_string(key));
    }

    return true, "";
}

apply_setting_text :: (config: *Format_Config, key: string, text: string, line: int) -> bool, string {
    doc, error := Toml.parse_toml(tprint("value = %", text));
    if error.kind != .NONE return false, tprint("line %: %", line, error.message);

    defer Toml.free_toml(*doc);
    if doc.root.members.count != 1 return false, tprint("line %: expected one setting value", line);

    return apply_toml_key(config, key, Toml.toml_get(doc.root, "value"), line);
}

apply_doc_key :: (config: *Format_Config, key: string, value: string, line_number: int) -> bool, string {
    if !is_doc_format_key(key) {
        array_add(*config.unknown_keys, copy_string(tprint("doc.format.%", key)));
        return true, "";
    }

    scratch: Format_Config;
    ok, message := apply_setting_text(*scratch, key, value, line_number);

    if !ok return false, message;

    array_add(*config.doc_format, Doc_Setting.{ copy_string(key), copy_string(value) });

    return true, "";
}

is_doc_format_key :: (key: string) -> bool {
    for DOC_FORMAT_KEYS if it == key return true;
    return false;
}

doc_example_config :: (config: Format_Config) -> Format_Config {
    inner := config;
    for config.doc_format apply_setting_text(*inner, it.key, it.value, 0);
    return inner;
}

// `line 0: message` -> `message`, for settings that did not come from a file.
without_line_prefix :: (message: string) -> string {
    if begins_with(message, "line 0: ") return slice(message, 8, message.count - 8);
    return message;
}

FORMAT_KEYS :: string.["indent_width", "case_indent", "case_body_indent", "max_blank_lines", "max_width", "brace_style", "blank_lines_between_items", "blank_line_after_declarations", "blank_lines_around_blocks", "blank_line_between_cases", "blank_line_before_comments", "blank_line_after_guards", "blank_line_before_return", "format_doc_examples", "ignore"];

// The settings `[doc.format]` can override: those of `[format]` that apply to code in a block.
DOC_FORMAT_KEYS :: string.["indent_width", "case_indent", "case_body_indent", "max_blank_lines", "max_width", "brace_style", "blank_lines_between_items", "blank_line_after_declarations", "blank_lines_around_blocks", "blank_line_between_cases", "blank_line_before_comments", "blank_line_after_guards", "blank_line_before_return"];

// The warning for an unknown `[format]` or `[doc.format]` setting `key` found in the config file
// `file`: it is ignored, and the nearest known key is named when there is one within a third of
// its length.
unknown_setting_note :: (key: string, file: string) -> string {
    prefix := "format.";
    keys: [] string = FORMAT_KEYS;
    name := key;

    if begins_with(key, "doc.format.") {
        prefix = "doc.format.";
        keys = DOC_FORMAT_KEYS;
        name = slice(key, prefix.count, key.count - prefix.count);
    } else if begins_with(key, "format.") {
        name = slice(key, prefix.count, key.count - prefix.count);
    }

    path := tprint("%%", prefix, name);

    note := tprint("unknown setting `%` in % (ignored)", path, file);

    limit := max(1, name.count / 3);
    best := -1;
    best_distance := limit + 1;

    for keys {
        distance := edit_distance(name, it);

        if distance < best_distance {
            best = it_index;
            best_distance = distance;
        }
    }

    if best >= 0 return tprint("%; did you mean `%%`?", note, prefix, keys[best]);

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

        memcpy(previous.data, current.data, (b.count + 1) * size_of(s64));
    }

    return previous[b.count];
}

parse_count :: (value: *Toml.Toml_Value, result: *int, low: int, high: int) -> bool {
    if value.kind != .INTEGER || value.integer < low || value.integer > high return false;
    result.* = value.integer;
    return true;
}

parse_flag :: (value: *Toml.Toml_Value, result: *bool) -> bool {
    if value.kind != .BOOL return false;
    result.* = value.boolean;
    return true;
}
