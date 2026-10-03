is_tbd :: (file_content: string) -> bool {
    reader := Tbd_Reader.{ content = file_content };
    if file_content.count >= 3 && file_content[0] == 0xef && file_content[1] == 0xbb && file_content[2] == 0xbf reader.cursor = 3;

    while reader.cursor < file_content.count {
        line := tbd_trim(tbd_line(*reader));
        if line.count == 0 || line[0] == 35 continue;
        return tbd_starts(line, "--- !tapi-tbd");
    }

    return false;
}

parse_tbd_file :: (path: string) -> success: bool, [] Text_Based_Dylib {
    content, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};
    defer B.free(content);
    return parse_tbd(content, path);
}

parse_tbd :: (file_content: string, path := "(unspecified file)") -> success: bool, [] Text_Based_Dylib {
    if !tbd_utf8_valid(file_content) {
        B.print("TBD %: invalid UTF-8 input\n", path);
        return false, .{};
    }

    reader := Tbd_Reader.{ content = file_content };
    if file_content.count >= 3 && file_content[0] == 0xef && file_content[1] == 0xbb && file_content[2] == 0xbf reader.cursor = 3;

    documents: [..] Text_Based_Dylib;
    transferred := false;

    defer if !transferred {
        for documents tbd_free_document(it);
        B.array_free(documents);
    }

    while reader.cursor < file_content.count {
        start := reader.cursor;
        line := tbd_trim(tbd_line(*reader));

        if line.count == 0 || line[0] == 35 continue;

        reader.cursor = start;
        ok, document := tbd_document(*reader);
        if !ok {
            B.print("TBD %: invalid or unsupported input at byte %\n", path, reader.cursor);
            return false, .{};
        }

        B.array_add(*documents, document);
    }

    if documents.count == 0 {
        B.print("TBD %: no documents\n", path);
        return false, .{};
    }

    transferred = true;

    return true, .{ documents.count, documents.data };
}

#scope_file
B :: #import "Basic";
F :: #import "File";

Tbd_Reader :: struct {
    content: string;
    cursor: s64;
}

tbd_utf8_valid :: (text: string) -> bool {
    if text.count < 0 return false;

    cursor: s64;
    while cursor < text.count {
        first := text[cursor];

        cursor += 1;

        if first < 128 {
            if first < 32 && first != 9 && first != 10 && first != 13 return false;
            continue;
        }

        trailing: s64;
        minimum: u32;
        scalar: u32;

        if first >= 0xc2 && first <= 0xdf {
            trailing = 1;
            minimum = 0x80;
            scalar = first & 31;
        } else if first >= 0xe0 && first <= 0xef {
            trailing = 2;
            minimum = 0x800;
            scalar = first & 15;
        } else if first >= 0xf0 && first <= 0xf4 {
            trailing = 3;
            minimum = 0x10000;
            scalar = first & 7;
        } else return false;

        if trailing > text.count - cursor return false;

        for 0..trailing - 1 {
            lane := text[cursor];
            cursor += 1;
            if lane < 0x80 || lane > 0xbf return false;
            scalar = (scalar << 6) | (lane & 63);
        }

        if scalar < minimum || scalar > 0x10ffff || (scalar >= 0xd800 && scalar <= 0xdfff) return false;
    }

    return true;
}

tbd_document :: (reader: *Tbd_Reader) -> bool, Text_Based_Dylib {
    result: Text_Based_Dylib;
    groups: [..] Target_Exports;
    archs: [] string;
    platform: string;
    root_keys: [..] string;
    group_keys: [..] string;
    transferred := false;

    defer tbd_free_strings(archs);
    defer B.free(platform);
    defer B.array_free(root_keys);
    defer B.array_free(group_keys);

    defer if !transferred {
        tbd_free_document(result);
        for groups tbd_free_group(it);
        B.array_free(groups);
    }

    header := tbd_header(tbd_trim(tbd_line(reader)));
    if header == "--- !tapi-tbd" || header == "--- !tapi-tbd-v4" result.tbd_version = 4;
    else if header == "--- !tapi-tbd-v3" result.tbd_version = 3;
    else return false, .{};

    context_kind: s32;
    group_index: s64 = -1;
    group_indent: s64 = -1;

    while reader.cursor < reader.content.count {
        start := reader.cursor;
        raw := tbd_line(reader);
        text := tbd_trim(raw);

        if text.count == 0 || text[0] == 35 continue;

        if tbd_starts(text, "---") {
            reader.cursor = start;
            break;
        }

        if tbd_header(text) == "..." break;

        indent: s64;
        while indent < raw.count && raw[indent] == 32 indent += 1;
        if indent < raw.count && raw[indent] == 9 return false, .{};

        dash := tbd_starts(text, "- ");
        if dash text = tbd_trim(.{ text.count - 2, text.data + 2 });

        colon: s64;
        while colon < text.count && text[colon] != 58 {
            c := text[colon];
            if !((c >= 97 && c <= 122) || (c >= 65 && c <= 90) || (c >= 48 && c <= 57) || c == 45 || c == 95) return false, .{};
            colon += 1;
        }

        if colon == 0 || colon == text.count return false, .{};

        key: string = .{ colon, text.data };

        reader.cursor = cast(s64) (text.data - reader.content.data) + colon + 1;
        ok, kind, values := tbd_value(reader);
        if !ok return false, .{};

        defer tbd_free_strings(values);

        if indent == 0 {
            if dash || tbd_seen(root_keys, key) return false, .{};

            B.array_add(*root_keys, key);
            context_kind = 0;
            group_index = -1;
            group_indent = -1;

            if key == "exports" || key == "reexported-libraries" || key == "reexports" {
                if kind == 2 && values.count == 0 continue;
                if kind != 0 return false, .{};

                context_kind = 1;
                if key == "reexported-libraries" context_kind = 2;
                if key == "reexports" context_kind = 3;
            } else if key == "targets" || key == "archs" {
                if kind == 0 {
                    ok, values = tbd_block_list(reader, indent);
                    if !ok return false, .{};
                    kind = 2;
                }

                if kind != 2 return false, .{};
                if key == "targets" result.targets = values;
                else archs = values;

                values = .{};
            } else if key == "install-name" {
                if kind != 1 || values[0].count == 0 return false, .{};
                result.install_name = values[0];
                B.array_free(values);
                values = .{};
            } else if key == "tbd-version" {
                if kind != 1 || (result.tbd_version == 4 && values[0] != "4") || (result.tbd_version == 3 && values[0] != "3") return false, .{};
            } else if key == "platform" {
                if kind != 1 return false, .{};
                platform = values[0];
                B.array_free(values);
                values = .{};
            } else if kind == 0 context_kind = 4; // Validated but unrepresented metadata block.
        } else {
            if context_kind == 0 return false, .{};
            if context_kind == 4 continue;

            if dash {
                if group_indent >= 0 && group_indent != indent return false, .{};

                group_indent = indent;
                B.array_add(*groups, Target_Exports.{});
                group_index = groups.count - 1;
                B.array_free(group_keys);
                group_keys = .{};
            } else if group_index < 0 || indent <= group_indent return false, .{};

            if tbd_seen(group_keys, key) return false, .{};

            B.array_add(*group_keys, key);
            group := *groups[group_index];
            is_names := key == "symbols" || key == "weak-symbols" || key == "weak-def-symbols" || key == "thread-local-symbols" || key == "objc-classes" || key == "objc-ivars" || key == "objc-eh-types";

            if key == "targets" || key == "archs" || key == "libraries" || key == "re-exports" || is_names {
                if kind == 0 {
                    ok, values = tbd_block_list(reader, indent);
                    if !ok return false, .{};
                    kind = 2;
                }

                if kind != 2 return false, .{};
                for values if it.count == 0 return false, .{};

                if key == "targets" || key == "archs" {
                    if group.targets.data return false, .{};
                    group.targets = values;
                } else if key == "libraries" || key == "re-exports" tbd_merge(*group.re_exports, values);
                else {
                    destination := *group.symbols;
                    if context_kind == 3 destination = *group.re_exports;
                    if key == "objc-classes" tbd_merge(destination, values, "_OBJC_CLASS_$_", "_OBJC_METACLASS_$_");
                    else if key == "objc-ivars" tbd_merge(destination, values, "_OBJC_IVAR_$_");
                    else if key == "objc-eh-types" tbd_merge(destination, values, "_OBJC_EHTYPE_$_");
                    else tbd_merge(destination, values);
                }

                values = .{};
            }
        }
    }

    if result.install_name.count == 0 return false, .{};

    if result.tbd_version == 3 {
        if result.targets.count != 0 || archs.count == 0 || platform.count == 0 return false, .{};

        result.targets = archs;
        archs = .{};
        if !tbd_map_targets(*result.targets, platform) return false, .{};
        for *groups if !tbd_map_targets(*it.targets, platform) return false, .{};
    } else if !tbd_seen(root_keys, "tbd-version") || archs.count != 0 || platform.count != 0 return false, .{};

    if result.targets.count == 0 return false, .{};

    for target: result.targets {
        if !tbd_target(target) return false, .{};
        for 0..it_index - 1 if result.targets[it] == target return false, .{};
    }

    for group: groups {
        if group.targets.count == 0 return false, .{};
        for group.targets if !tbd_seen(result.targets, it) return false, .{};
    }

    result.exports = .{ groups.count, groups.data };
    transferred = true;

    return true, result;
}

tbd_line :: (reader: *Tbd_Reader) -> string {
    start := reader.cursor;

    while reader.cursor < reader.content.count && reader.content[reader.cursor] != 10 reader.cursor += 1;

    result: string = .{ reader.cursor - start, reader.content.data + start };

    if reader.cursor < reader.content.count reader.cursor += 1;

    return result;
}

tbd_trim :: (text: string) -> string {
    while text.count > 0 && (text[0] == 32 || text[0] == 9 || text[0] == 13) {
        text.data += 1;
        text.count -= 1;
    }

    while text.count > 0 && (text[text.count - 1] == 32 || text[text.count - 1] == 9 || text[text.count - 1] == 13) text.count -= 1;

    return text;
}

tbd_starts :: (text: string, prefix: string) -> bool {
    if text.count < prefix.count return false;
    for prefix if text[it_index] != it return false;
    return true;
}

tbd_seen :: (keys: [] string, key: string) -> bool {
    for keys if it == key return true;
    return false;
}

tbd_header :: (text: string) -> string {
    if tbd_starts(text, "---") || tbd_starts(text, "...") {
        for text if it == 35 return tbd_trim(.{ it_index, text.data });
    }

    return text;
}

tbd_space :: (reader: *Tbd_Reader, multiline: bool) {
    while reader.cursor < reader.content.count {
        c := reader.content[reader.cursor];
        if c == 32 || c == 9 || c == 13 || (multiline && c == 10) reader.cursor += 1;
        else if c == 35 {
            while reader.cursor < reader.content.count && reader.content[reader.cursor] != 10 reader.cursor += 1;
            if !multiline return;
        } else return;
    }
}

tbd_quoted :: (reader: *Tbd_Reader, quote: u8) -> bool, string {
    // Normalize YAML escape spellings to the shared lexer's Unicode escapes.
    // In particular YAML \xNN denotes a Unicode scalar, not a raw byte.
    normalized: [..] u8;

    defer B.array_free(normalized);
    B.array_add(*normalized, quote);
    reader.cursor += 1;
    while reader.cursor < reader.content.count {
        c := reader.content[reader.cursor];

        reader.cursor += 1;
        B.array_add(*normalized, c);

        if c == quote {
            if quote == 39 && reader.cursor < reader.content.count && reader.content[reader.cursor] == 39 {
                B.array_add(*normalized, quote);
                reader.cursor += 1;
                continue;
            }

            break;
        }

        if c == 92 && quote == 34 && reader.cursor < reader.content.count {
            escape := reader.content[reader.cursor];

            reader.cursor += 1;
            expansion: string;
            if escape == #char "x" expansion = "u00";
            if escape == #char "a" expansion = "u0007";
            if escape == #char "b" expansion = "u0008";
            if escape == #char "v" expansion = "u000b";
            if escape == #char "f" expansion = "u000c";
            if escape == #char "e" expansion = "u001b";
            if escape == #char "N" expansion = "u0085";
            if escape == #char "_" expansion = "u00a0";
            if escape == #char "L" expansion = "u2028";
            if escape == #char "P" expansion = "u2029";

            if expansion.count != 0 {
                for expansion B.array_add(*normalized, it);
            } else if escape == 47 || escape == 32 {
                normalized.count -= 1;
                B.array_add(*normalized, escape);
            } else B.array_add(*normalized, escape);
        }
    }

    lexer: Lexer;

    set_input_from_string(*lexer, .{ normalized.count, normalized.data });
    token := make_string_constant(*lexer, quote);
    if token.type != .STRING || lexer.input_cursor != normalized.count return false, "";

    return true, B.copy_string(token.string_value);
}

tbd_scalar :: (reader: *Tbd_Reader, flow: bool) -> bool, string {
    tbd_space(reader, flow);
    if reader.cursor >= reader.content.count return false, "";

    c := reader.content[reader.cursor];
    if c == 34 || c == 39 return tbd_quoted(reader, c);

    if c == 42 || c == 38 || c == 33 || c == 123 || c == 125 || c == 91 || c == 93 || c == 44 return false, "";

    start := reader.cursor;

    while reader.cursor < reader.content.count {
        c = reader.content[reader.cursor];
        comment := c == 35 && (reader.cursor == start || reader.content[reader.cursor - 1] == 32 || reader.content[reader.cursor - 1] == 9 || reader.content[reader.cursor - 1] == 13);
        if c == 10 || comment || (flow && (c == 44 || c == 93)) break;
        if c < 32 && c != 9 && c != 13 return false, "";
        if c == 91 || c == 123 || c == 125 || c == 34 || c == 39 return false, "";
        if c == 58 && reader.cursor + 1 < reader.content.count && reader.content[reader.cursor + 1] == 32 return false, "";

        reader.cursor += 1;
    }

    text := tbd_trim(.{ reader.cursor - start, reader.content.data + start });
    if text.count == 0 return false, "";

    return true, B.copy_string(text);
}

tbd_value :: (reader: *Tbd_Reader) -> bool, s32, [] string {
    values: [..] string;
    transferred := false;

    defer if !transferred {
        for values B.free(it);
        B.array_free(values);
    }

    tbd_space(reader, false);

    if reader.cursor >= reader.content.count || reader.content[reader.cursor] == 10 {
        if reader.cursor < reader.content.count reader.cursor += 1;
        return true, 0, .{};
    }

    kind: s32 = 1;

    if reader.content[reader.cursor] == 91 {
        kind = 2;
        reader.cursor += 1;
        tbd_space(reader, true);
        if reader.cursor >= reader.content.count return false, 0, .{};

        if reader.content[reader.cursor] != 93 {
            while true {
                ok, value := tbd_scalar(reader, true);
                if !ok return false, 0, .{};

                B.array_add(*values, value);
                tbd_space(reader, true);
                if reader.cursor >= reader.content.count return false, 0, .{};

                c := reader.content[reader.cursor];
                if c == 93 break;
                if c != 44 return false, 0, .{};

                reader.cursor += 1;
                tbd_space(reader, true);
                if reader.cursor >= reader.content.count return false, 0, .{};
                if reader.content[reader.cursor] == 93 break;
            }
        }

        reader.cursor += 1;
    } else {
        ok, value := tbd_scalar(reader, false);
        if !ok return false, 0, .{};
        B.array_add(*values, value);
    }

    tbd_space(reader, false);
    if reader.cursor < reader.content.count && reader.content[reader.cursor] != 10 return false, 0, .{};
    if reader.cursor < reader.content.count reader.cursor += 1;

    transferred = true;

    return true, kind, .{ values.count, values.data };
}

tbd_block_list :: (reader: *Tbd_Reader, parent_indent: s64) -> bool, [] string {
    values: [..] string;
    transferred := false;

    defer if !transferred {
        for values B.free(it);
        B.array_free(values);
    }

    while reader.cursor < reader.content.count {
        start := reader.cursor;
        raw := tbd_line(reader);
        text := tbd_trim(raw);

        if text.count == 0 || text[0] == 35 continue;

        indent: s64;
        while indent < raw.count && raw[indent] == 32 indent += 1;

        if indent <= parent_indent || !tbd_starts(text, "- ") {
            reader.cursor = start;
            break;
        }

        reader.cursor = cast(s64) (text.data - reader.content.data) + 2;
        ok, kind, item := tbd_value(reader);
        if !ok || kind != 1 {
            tbd_free_strings(item);
            return false, .{};
        }

        B.array_add(*values, item[0]);
        B.array_free(item);
    }

    transferred = true;

    return true, .{ values.count, values.data };
}

tbd_merge :: (destination: *[] string, incoming: [] string, prefix := "", second_prefix := "") {
    values: [..] string;
    for destination.* B.array_add(*values, it);

    for incoming {
        if prefix.count == 0 B.array_add(*values, it);
        else {
            B.array_add(*values, B.sprint("%%", prefix, it));
            if second_prefix.count != 0 B.array_add(*values, B.sprint("%%", second_prefix, it));
            B.free(it);
        }
    }

    B.array_free(destination.*);
    B.array_free(incoming);
    destination.* = .{ values.count, values.data };
}

tbd_map_targets :: (targets: *[] string, platform: string) -> bool {
    normalized := platform;
    if normalized == "macosx" normalized = "macos";
    if normalized != "macos" && normalized != "ios" && normalized != "tvos" && normalized != "watchos" && normalized != "bridgeos" && normalized != "maccatalyst" && normalized != "driverkit" && normalized != "zippered" return false;

    mapped: [..] string;

    for arch: targets.* {
        if arch != "x86_64" && arch != "x86_64h" && arch != "i386" && arch != "arm64" && arch != "arm64e" && arch != "armv7" && arch != "armv7s" && arch != "armv7k" && arch != "arm64_32" {
            for mapped B.free(it);
            B.array_free(mapped);
            return false;
        }

        target_platform := normalized;
        if target_platform == "zippered" {
            B.array_add(*mapped, B.sprint("%-macos", arch));
            B.array_add(*mapped, B.sprint("%-maccatalyst", arch));
        } else {
            if (arch == "i386" || arch == "x86_64" || arch == "x86_64h") && (normalized == "ios" || normalized == "tvos" || normalized == "watchos") target_platform = B.tprint("%-simulator", normalized);
            B.array_add(*mapped, B.sprint("%-%", arch, target_platform));
        }
    }

    tbd_free_strings(targets.*);
    targets.* = .{ mapped.count, mapped.data };

    return true;
}

tbd_target :: (target: string) -> bool {
    dash := false;

    for target {
        if it <= 32 || it == 127 return false;
        if it == 45 dash = true;
    }

    return dash && target.count > 2;
}

tbd_free_strings :: (values: [] string) {
    for values B.free(it);
    B.array_free(values);
}

tbd_free_group :: (group: Target_Exports) {
    tbd_free_strings(group.targets);
    tbd_free_strings(group.symbols);
    tbd_free_strings(group.re_exports);
}

tbd_free_document :: (document: Text_Based_Dylib) {
    tbd_free_strings(document.targets);
    B.free(document.install_name);
    for document.exports tbd_free_group(it);
    B.array_free(document.exports);
}
