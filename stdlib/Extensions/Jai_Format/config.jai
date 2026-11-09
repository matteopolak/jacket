// jacket.toml schema adaptation and ignore globs. The typed TOML layer validates whole files.
#scope_module

resolved_case_indent :: (config: Format_Config) -> int {
    if config.case_indent < 0 return config.indent_width;
    return config.case_indent;
}

resolved_case_body_indent :: (config: Format_Config) -> int {
    if config.case_body_indent < 0 return config.indent_width;
    return config.case_body_indent;
}

// The tables of a jacket.toml this module reads. Other tables are validated as TOML and
// ignored; the settings and their messages are the `@toml` notes of Format_Config.
Config_File :: struct {
    format: Config_Section; @toml(default, message="format must be a table")
    doc: Config_Doc; @toml(default, message="doc must be a table")
}

Config_Doc :: struct {
    format: Config_Doc_Section; @toml(default, message="doc.format must be a table")
}

// `[format]`: the settings plus every key the formatter does not know, in file order.
Config_Section :: struct {
    settings: Format_Config; @toml(flatten, default)
    unknown: [..] Toml.Toml_Unknown; @toml(unknown)
}

// `[doc.format]` keeps each key with its text as written; only the keys of DOC_FORMAT_KEYS are
// settings, and they are checked below against the same schema as `[format]`.
Config_Doc_Section :: struct {
    unknown: [..] Toml.Toml_Unknown; @toml(unknown)
}

// Validates the whole file once and reads `[format]` and `[doc.format]`. `[doc.format]` values
// keep their source text (a later pass applies them to code blocks), so they are checked here
// but stored raw.
parse_config_into :: (text: string, config: *Format_Config) -> bool, string {
    file: Config_File;
    error := Toml.toml_decode(text, *file);

    if error.kind != .NONE return false, tprint("line %: %", error.line, error.message);

    defer Toml.free_toml_value(*file);

    // Unknown keys of both tables are listed in file order.
    format := file.format.unknown;
    docs := file.doc.format.unknown;
    next_format := 0;
    next_docs := 0;

    while next_format < format.count || next_docs < docs.count {
        take_format := next_docs >= docs.count || (next_format < format.count && format[next_format].start < docs[next_docs].start);

        if take_format {
            entry := format[next_format];
            next_format += 1;
            array_add(*config.unknown_keys, ifx entry.header then tprint("format.%", entry.path) else copy_string(entry.path));

            continue;
        }

        entry := docs[next_docs];

        next_docs += 1;

        if entry.header || !is_doc_format_key(entry.path) {
            array_add(*config.unknown_keys, tprint("doc.format.%", entry.path));
            continue;
        }

        raw := slice(text, entry.value, entry.end - entry.value);
        ok, message := check_setting_text(entry.path, raw, config_line(text, entry.start));

        if !ok return false, message;

        array_add(*config.doc_format, Doc_Setting.{ copy_string(entry.path), copy_string(raw) });
    }

    // The settings move into the result; the rest of the decoded file is released.
    keep := config.unknown_keys;
    doc_keep := config.doc_format;
    root := config.root;

    config.* = file.format.settings;
    config.unknown_keys = keep;
    config.doc_format = doc_keep;
    config.root = root;
    file.format.settings = .{};

    return true, "";
}

config_line :: (text: string, offset: int) -> int {
    line := 1;
    for 0..offset - 1 if text[it] == 10 line += 1;
    return line;
}

// Decodes `key = text` as a `[format]` table. The key is one of the settings, so what
// comes back is that setting alone, validated by the schema.
decode_setting :: (key: string, text: string, line: int) -> Config_Section, bool, string {
    section: Config_Section;
    error := Toml.toml_decode(tprint("% = %\n", key, text), *section);

    if error.kind != .NONE return section, false, tprint("line %: %", line, error.message);

    return section, true, "";
}

check_setting_text :: (key: string, text: string, line: int) -> bool, string {
    section, ok, message := decode_setting(key, text, line);

    defer Toml.free_toml_value(*section);

    return ok, message;
}

// Sets one `[format]` setting from `key = text`, the value written as in the file.
apply_setting_text :: (config: *Format_Config, key: string, text: string, line: int) -> bool, string {
    section, ok, message := decode_setting(key, text, line);

    defer Toml.free_toml_value(*section);

    if !ok return false, message;

    if section.unknown.count {
        array_add(*config.unknown_keys, copy_string(key));
        return true, "";
    }

    move_setting(config, *section.settings, key);

    return true, "";
}

// Copies the setting `key` from a decoded section: the member of Format_Config with that name.
// `ignore` is the one setting that owns memory; the globs it replaces are released and the
// decoded ones move over.
move_setting :: (config: *Format_Config, from: *Format_Config, key: string) {
    if key == "ignore" {
        for config.ignore free(it);

        array_free(config.ignore);
        config.ignore = from.ignore;
        from.ignore = .{};

        return;
    }

    member, offset := get_field(cast(*Type_Info_Struct) type_info(Format_Config), key);

    if !member return;

    memcpy(cast(*u8) config + offset, cast(*u8) from + offset, member.type.runtime_size);
}

apply_doc_key :: (config: *Format_Config, key: string, value: string, line_number: int) -> bool, string {
    if !is_doc_format_key(key) {
        array_add(*config.unknown_keys, copy_string(tprint("doc.format.%", key)));
        return true, "";
    }

    ok, message := check_setting_text(key, value, line_number);

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
