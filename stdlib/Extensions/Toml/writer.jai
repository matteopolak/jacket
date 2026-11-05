#scope_export
// Writes a deterministic semantic TOML document. Comments/layout are not preserved. All nested
// tables are written inline, retaining empty tables and mixed arrays without header ambiguity.
// Example: text, error := write_toml(doc.root); defer free(text);
write_toml :: (root: *Toml_Value, options := Toml_Options.{}) -> string, Toml_Error {
    writer: Toml_Writer;
    writer.options = options;

    if !toml_options_valid(options) {
        toml_write_fail(*writer, .LIMIT, "limits must be nonnegative");
        return "", writer.error;
    }

    defer free_buffers(*writer.builder);
    defer free_buffers(*writer.float_scratch);
    defer array_free(writer.path);

    if !root || root.kind != .TABLE {
        toml_write_fail(*writer, .OUTPUT, "document root must be a table");
        return "", writer.error;
    }

    if options.max_nodes < 1 {
        toml_write_fail(*writer, .LIMIT, "writer node limit exceeded");
        return "", writer.error;
    }

    writer.nodes = 1;
    toml_write_table(*writer, root, 0, false);
    if writer.error.kind != .NONE return "", writer.error;
    if writer.builder.failed return "", .{ kind = .OUTPUT, message = "output allocation failed" };

    return builder_to_string(*writer.builder), .{};
}

#scope_module
Toml_Writer :: struct {
    builder: String_Builder;
    float_scratch: String_Builder;
    options: Toml_Options;
    error: Toml_Error;
    bytes: int;
    nodes: int;
    path: [..] Toml_Path_Part;
}

toml_emit :: (w: *Toml_Writer, text: string) {
    if w.error.kind != .NONE return;

    if text.count > w.options.max_output_bytes - w.bytes {
        toml_write_fail(w, .LIMIT, "output byte limit exceeded");
        return;
    }

    append(*w.builder, text);
    w.bytes += text.count;
}

toml_write_string :: (w: *Toml_Writer, text: string) {
    if text.count > w.options.max_string_bytes {
        toml_write_fail(w, .LIMIT, "writer string byte limit exceeded");
        return;
    }

    invalid_at, _ := toml_utf8_error(text);
    if invalid_at >= 0 {
        toml_write_fail(w, .OUTPUT, "string is not valid UTF-8");
        return;
    }

    toml_emit(w, "\"");

    plain_start := 0;

    for text {
        c := it;
        if c != 34 && c != 92 && c >= 32 && c != 127 continue;

        toml_emit(w, slice(text, plain_start, it_index - plain_start));
        if w.error.kind != .NONE return;

        if c == 34 toml_emit(w, "\\\"");
        else if c == 92 toml_emit(w, "\\\\");
        else if c < 32 || c == 127 {
            bytes: [6] u8;
            bytes[0] = 92;
            bytes[1] = 117;
            bytes[2] = 48;
            bytes[3] = 48;
            hex := "0123456789abcdef";

            bytes[4] = hex[c >> 4];
            bytes[5] = hex[c & 15];
            toml_emit(w, string.{ 6, bytes.data });
        }

        plain_start = it_index + 1;
    }

    toml_emit(w, slice(text, plain_start, text.count - plain_start));
    toml_emit(w, "\"");
}

// TOML's canonical signed decimal is bounded to 20 bytes. A local buffer avoids general
// display formatting and temporary-storage results; emit copies the complete slice here.
toml_write_integer :: (w: *Toml_Writer, value: s64) {
    bytes: [20] u8 = ---;
    start := bytes.count;
    magnitude := ifx value < 0 then cast(u64) (-(value + 1)) + 1 else cast(u64) value;

    while true {
        start -= 1;
        bytes[start] = cast(u8) (48 + magnitude % 10);
        magnitude /= 10;
        if magnitude == 0 break;
    }

    if value < 0 {
        start -= 1;
        bytes[start] = 45;
    }

    toml_emit(w, string.{ bytes.count - start, bytes.data + start });
}

toml_write_value :: (w: *Toml_Writer, node: *Toml_Value, depth: int) {
    if w.error.kind != .NONE return;

    w.nodes += 1;
    if w.nodes > w.options.max_nodes {
        toml_write_fail(w, .LIMIT, "writer node limit exceeded");
        return;
    }

    if !node {
        toml_write_fail(w, .OUTPUT, "null is not a TOML value");
        return;
    }

    if depth > w.options.max_depth {
        toml_write_fail(w, .LIMIT, "writer nesting limit exceeded");
        return;
    }

    if node.kind == .STRING toml_write_string(w, node.text);
    else if node.kind == .INTEGER toml_write_integer(w, node.integer);
    else if node.kind == .BOOL toml_emit(w, ifx node.boolean then "true" else "false");
    else if node.kind == .FLOAT {
        bits := cast,force(u64) node.real;
        if (bits & 0x7fffffffffffffff) > 0x7ff0000000000000 toml_emit(w, "nan");
        else if (bits & 0x7fffffffffffffff) == 0x7ff0000000000000 toml_emit(w, ifx (bits >> 63) != 0 then "-inf" else "inf");
        else {
            format: FormatFloat;
            format.value = node.real;
            format.width = -1;
            format.mode = .SCIENTIFIC;
            format.trailing_width = 16;
            format.zero_removal = .NO;
            format.thousand_separators = false;
            format.decimal_separator_character = 46;
            format.exponent_character = 101;
            format.positive_number_prefix = .NONE;
            print_float(*w.float_scratch, format);

            if w.float_scratch.failed {
                toml_write_fail(w, .OUTPUT, "float formatting allocation failed");
                return;
            }

            // Keep conversion scratch separate: output admission still precedes every copy.
            buffer := get_base_buffer(*w.float_scratch);
            while buffer {
                toml_emit(w, string.{ buffer.count, get_buffer_data(buffer) });
                buffer = buffer.next;
            }

            reset(*w.float_scratch);
        }
    } else if node.kind == .ARRAY {
        toml_emit(w, "[");

        for node.items {
            if it_index toml_emit(w, ", ");

            saved_path := w.path.count;

            array_add(*w.path, Toml_Path_Part.{ index = it_index });
            toml_write_value(w, it, depth + 1);
            w.path.count = saved_path;
            if w.error.kind != .NONE return;
        }

        toml_emit(w, "]");
    } else if node.kind == .TABLE {
        toml_write_table(w, node, depth, true);
    } else {
        // Temporal output uses canonical separators/seconds and preserves fraction/offset kind.
        t := node.temporal;

        if node.kind != .LOCAL_DATE && node.kind != .LOCAL_TIME && node.kind != .LOCAL_DATETIME && node.kind != .OFFSET_DATETIME {
            toml_write_fail(w, .OUTPUT, "unknown TOML value kind");
            return;
        }

        if (node.kind != .LOCAL_TIME && !toml_valid_date(t)) || (node.kind != .LOCAL_DATE && !toml_valid_time(t)) {
            toml_write_fail(w, .OUTPUT, "invalid constructed temporal value");
            return;
        }

        if node.kind == .OFFSET_DATETIME && (t.offset_minutes < -1439 || t.offset_minutes > 1439 || (t.negative_zero_offset && t.offset_minutes != 0)) {
            toml_write_fail(w, .OUTPUT, "invalid constructed UTC offset");
            return;
        }

        if t.fraction.count > w.options.max_string_bytes {
            toml_write_fail(w, .LIMIT, "writer fraction byte limit exceeded");
            return;
        }

        for t.fraction if !toml_digit(it) {
            toml_write_fail(w, .OUTPUT, "invalid constructed fraction");
            return;
        }

        bytes: [20] u8 = ---;
        prefix := ([] u8).{ bytes.count, bytes.data };
        count := 0;

        if node.kind != .LOCAL_TIME {
            toml_put_decimal(prefix, 0, t.year, 4);
            bytes[4] = 45;
            toml_put_decimal(prefix, 5, t.month, 2);
            bytes[7] = 45;
            toml_put_decimal(prefix, 8, t.day, 2);
            count = 10;
        }

        if node.kind != .LOCAL_DATE {
            if count {
                bytes[count] = 84;
                count += 1;
            }

            toml_put_decimal(prefix, count, t.hour, 2);
            bytes[count + 2] = 58;
            toml_put_decimal(prefix, count + 3, t.minute, 2);
            bytes[count + 5] = 58;
            toml_put_decimal(prefix, count + 6, t.second, 2);
            count += 8;

            if t.fraction.count {
                bytes[count] = 46;
                count += 1;
            }
        }

        toml_emit(w, string.{ count, bytes.data });
        if node.kind != .LOCAL_DATE && t.fraction.count toml_emit(w, t.fraction);

        if node.kind == .OFFSET_DATETIME {
            offset := ifx t.offset_minutes < 0 then -t.offset_minutes else t.offset_minutes;
            suffix: [6] u8 = ---;
            offset_bytes := ([] u8).{ suffix.count, suffix.data };

            suffix[0] = ifx t.offset_minutes < 0 || t.negative_zero_offset then 45 else 43;
            toml_put_decimal(offset_bytes, 1, offset / 60, 2);
            suffix[3] = 58;
            toml_put_decimal(offset_bytes, 4, offset % 60, 2);
            toml_emit(w, string.{ suffix.count, suffix.data });
        }
    }
}

// Callers validate fixed civil fields first; every requested digit fits its width.
toml_put_decimal :: inline (bytes: [] u8, offset: int, value: int, width: int) {
    remaining := value;

    for i: 0..width - 1 {
        bytes[offset + width - 1 - i] = cast(u8) (48 + remaining % 10);
        remaining /= 10;
    }
}

// A cache is only a witness: current keys must form a strict in-order sequence and cover
// every slot exactly once. Bounded pushes and a fixed stack make corrupt graphs fail closed.
// The caller falls back to ordinary validation when any part of this proof fails.
toml_index_proves_unique :: (table: *Toml_Value) -> bool {
    index := table.internal_lookup;
    count := table.members.count;

    if !index || index.nodes.count != count return false;
    if count == 0 return index.root == -1;
    if index.root < 0 || index.root >= count return false;

    stack: [128] int;
    top := 0;
    pushed := 0;
    visited := 0;
    previous := -1;
    current := index.root;

    while current >= 0 || top > 0 {
        while current >= 0 {
            if current >= count || pushed >= count || top >= stack.count return false;

            stack[top] = current;
            top += 1;
            pushed += 1;
            current = index.nodes[current].left;
            if current < -1 return false;
        }

        top -= 1;
        slot := stack[top];
        if previous >= 0 && toml_key_compare(table.members[previous].key, table.members[slot].key) >= 0 return false;

        previous = slot;
        visited += 1;
        current = index.nodes[slot].right;
        if current < -1 || current >= count return false;
    }

    return visited == count && pushed == count;
}

toml_write_table :: (w: *Toml_Writer, node: *Toml_Value, depth: int, inline_table: bool) {
    keys: Toml_Index;
    // Do not pre-walk more immediate members than the remaining node quota permits.
    // Incremental fallback preserves the existing first-error path for smaller limits.
    proven := node.members.count <= w.options.max_nodes - w.nodes && toml_index_proves_unique(node);
    tiny := node.members.count <= 16;

    defer array_free(keys.nodes);
    if inline_table toml_emit(w, "{");

    for node.members {
        saved_path := w.path.count;

        array_add(*w.path, Toml_Path_Part.{ name = it.key });

        if !proven {
            duplicate := -1;
            hint: Toml_Insert_Hint;

            if tiny {
                for previous: 0..it_index - 1 {
                    if toml_key_compare(node.members[previous].key, it.key) == 0 {
                        duplicate = previous;
                        break;
                    }
                }
            } else duplicate = toml_probe_index(node, *keys, it.key, *hint);

            if duplicate >= 0 {
                toml_write_fail(w, .OUTPUT, "duplicate constructed table key");
                return;
            }

            if !tiny && toml_tree_add(node, *keys, it_index, *hint, false) >= 0 {
                toml_write_fail(w, .OUTPUT, "duplicate constructed table key");
                return;
            }
        }

        if inline_table && it_index toml_emit(w, ", ");

        toml_write_string(w, it.key);
        toml_emit(w, " = ");
        toml_write_value(w, it.value, depth + 1);
        w.path.count = saved_path;
        if !inline_table toml_emit(w, "\n");
        if w.error.kind != .NONE return;
    }

    if inline_table toml_emit(w, "}");
}

toml_write_fail :: (w: *Toml_Writer, kind: Toml_Error_Kind, message: string) {
    if w.error.kind != .NONE return;

    w.error.kind = kind;
    w.error.message = message;
    reader: Toml_Reader;
    reader.path = w.path;
    toml_render_path(*reader, *w.error);
}
