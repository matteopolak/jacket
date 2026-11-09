#scope_export
// Writes a deterministic semantic TOML document. Comments/layout are not preserved. A table
// inside a table is written under a `[header]` and a nonempty array of tables under `[[header]]`,
// after the table's own `key = value` lines (an empty table is a bare `[header]`, as in the typed
// encoder). A table inside any other array stays inline.
// Example: text, error := write_toml(doc.root); defer free(text);
write_toml :: (root: *Toml_Value, options := Toml_Options.{}) -> string, Toml_Error {
    writer: Toml_Writer;
    writer.options = options;
    nanoseconds, _ := to_nanoseconds(current_time_monotonic());

    writer.seed = cast(u64) (nanoseconds ^ cast(s64) cast(*void) *writer);

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
    depth: int;
    seed: u64; // randomizes the key hash of duplicate checks
}

// True when `count` more bytes fit the output limit and nothing has failed. A key or a value
// asks once for everything it will write, then adds the pieces with `toml_put`.
toml_room :: (w: *Toml_Writer, count: int) -> bool {
    if w.error.kind != .NONE return false;

    if count > w.options.max_output_bytes - w.bytes {
        toml_write_fail(w, .LIMIT, "output byte limit exceeded");
        return false;
    }

    return true;
}

// Adds bytes that `toml_room` has already admitted.
toml_put :: (w: *Toml_Writer, text: string) {
    append(*w.builder, text);
    w.bytes += text.count;
}

toml_emit :: (w: *Toml_Writer, text: string) {
    if toml_room(w, text.count) toml_put(w, text);
}

toml_write_string :: (w: *Toml_Writer, text: string) {
    if text.count > w.options.max_string_bytes {
        toml_write_fail(w, .LIMIT, "writer string byte limit exceeded");
        return;
    }

    // One scan sizes the output: each quote or backslash gains a byte, each control character
    // five. Plain ASCII needs neither escapes nor a UTF-8 proof.
    extra := 0;
    ascii := true;

    for text {
        if it == 34 || it == 92 extra += 1;
        else if it < 32 || it == 127 extra += 5;
        else if it >= 128 ascii = false;
    }

    if !ascii {
        invalid_at, _ := toml_utf8_error(text);
        if invalid_at >= 0 {
            toml_write_fail(w, .OUTPUT, "string is not valid UTF-8");
            return;
        }
    }

    if !toml_room(w, text.count + extra + 2) return;

    toml_put(w, "\"");

    if !extra {
        toml_put(w, text);
        toml_put(w, "\"");
        return;
    }

    plain_start := 0;

    for text {
        c := it;
        if c != 34 && c != 92 && c >= 32 && c != 127 continue;

        toml_put(w, slice(text, plain_start, it_index - plain_start));

        if c == 34 toml_put(w, "\\\"");
        else if c == 92 toml_put(w, "\\\\");
        else {
            bytes: [6] u8;
            bytes[0] = 92;
            bytes[1] = 117;
            bytes[2] = 48;
            bytes[3] = 48;
            hex := "0123456789abcdef";

            bytes[4] = hex[c >> 4];
            bytes[5] = hex[c & 15];
            toml_put(w, string.{ 6, bytes.data });
        }

        plain_start = it_index + 1;
    }

    toml_put(w, slice(text, plain_start, text.count - plain_start));
    toml_put(w, "\"");
}

// A bare key is nonempty ASCII only. This pass proves UTF-8 validity as well as syntax;
// other keys retain the full checked quoted-string path. Values always stay quoted. `suffix`
// (` = `, or nothing for a header) is admitted together with a bare key.
toml_write_key :: (w: *Toml_Writer, text: string, suffix := "") {
    if text.count > w.options.max_string_bytes {
        toml_write_fail(w, .LIMIT, "writer string byte limit exceeded");
        return;
    }

    bare := text.count > 0;

    for text {
        c := it;
        if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 95 || c == 45 continue;
        bare = false;
        break;
    }

    if bare {
        if !toml_room(w, text.count + suffix.count) return;

        toml_put(w, text);
        toml_put(w, suffix);

        return;
    }

    toml_write_string(w, text);
    toml_emit(w, suffix);
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

// Counts `node` against the node and depth limits; false once the writer has failed.
toml_count_node :: (w: *Toml_Writer, node: *Toml_Value, depth: int) -> bool {
    if w.error.kind != .NONE return false;

    w.nodes += 1;
    if w.nodes > w.options.max_nodes {
        toml_write_fail(w, .LIMIT, "writer node limit exceeded");
        return false;
    }

    if !node {
        toml_write_fail(w, .OUTPUT, "null is not a TOML value");
        return false;
    }

    if depth > w.options.max_depth {
        toml_write_fail(w, .LIMIT, "writer nesting limit exceeded");
        return false;
    }

    return true;
}

// Counts one scalar against the node limit; false once the writer has failed.
toml_count_leaf :: (w: *Toml_Writer) -> bool {
    if w.error.kind != .NONE return false;

    w.nodes += 1;

    if w.nodes > w.options.max_nodes {
        toml_write_fail(w, .LIMIT, "writer node limit exceeded");
        return false;
    }

    return true;
}

toml_write_value :: (w: *Toml_Writer, node: *Toml_Value, depth: int) {
    if !toml_count_node(w, node, depth) return;

    if node.kind == .STRING toml_write_string(w, node.text);
    else if node.kind == .INTEGER toml_write_integer(w, node.integer);
    else if node.kind == .BOOL toml_emit(w, ifx node.boolean then "true" else "false");
    else if node.kind == .FLOAT toml_write_float(w, node.real);
    else if node.kind == .ARRAY {
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

        // Date or time prefix, fraction and offset suffix are admitted together.
        total := ifx node.kind == .LOCAL_DATE then 10 else ifx node.kind == .LOCAL_TIME then 8 else 19;

        if node.kind != .LOCAL_DATE && t.fraction.count total += 1 + t.fraction.count;
        if node.kind == .OFFSET_DATETIME total += 6;
        if !toml_room(w, total) return;

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

        toml_put(w, string.{ count, bytes.data });
        if node.kind != .LOCAL_DATE && t.fraction.count toml_put(w, t.fraction);

        if node.kind == .OFFSET_DATETIME {
            offset := ifx t.offset_minutes < 0 then -t.offset_minutes else t.offset_minutes;
            suffix: [6] u8 = ---;
            offset_bytes := ([] u8).{ suffix.count, suffix.data };

            suffix[0] = ifx t.offset_minutes < 0 || t.negative_zero_offset then 45 else 43;
            toml_put_decimal(offset_bytes, 1, offset / 60, 2);
            suffix[3] = 58;
            toml_put_decimal(offset_bytes, 4, offset % 60, 2);
            toml_put(w, string.{ suffix.count, suffix.data });
        }
    }
}

// The shortest digits that read back as the same binary64 or, for a float32, the same binary32
// (signed zeros and subnormals included), laid out as a plain decimal when the exponent is small and kept scientific
// otherwise. Every spelling has a fraction or an exponent, so it stays a float.
toml_write_float :: (w: *Toml_Writer, value: $T) {
    // Widening is exact, so the special values are told apart the same way for both widths.
    bits := cast,force(u64) cast(float64) value;

    if (bits & 0x7fffffffffffffff) > 0x7ff0000000000000 {
        toml_emit(w, "nan");
        return;
    }

    if (bits & 0x7fffffffffffffff) == 0x7ff0000000000000 {
        toml_emit(w, ifx (bits >> 63) != 0 then "-inf" else "inf");
        return;
    }

    format: FormatFloat;
    format.value = value;
    format.width = -1;
    format.mode = .SHORTEST;
    format.thousand_separators = false;
    format.decimal_separator_character = 46;
    format.exponent_character = 101;
    format.positive_number_prefix = .NONE;
    print_float(*w.float_scratch, format);

    if w.float_scratch.failed {
        toml_write_fail(w, .OUTPUT, "float formatting allocation failed");
        return;
    }

    // At most 24 bytes: sign, 17 digits, point, exponent. Conversion scratch stays separate so
    // output admission still precedes every copy.
    raw: [32] u8 = ---;
    raw_count := 0;
    buffer := get_base_buffer(*w.float_scratch);

    while buffer {
        for 0..buffer.count - 1 {
            if raw_count < raw.count {
                raw[raw_count] = get_buffer_data(buffer)[it];
                raw_count += 1;
            }
        }

        buffer = buffer.next;
    }

    reset(*w.float_scratch);

    negative := raw_count > 0 && raw[0] == 45;
    digits: [24] u8 = ---;
    digit_count := 0;
    exponent := 0;
    point := -1;
    cursor := ifx negative then 1 else 0;
    has_exponent := false;

    while cursor < raw_count {
        c := raw[cursor];

        if c == 46 point = digit_count;
        else if c == 101 {
            has_exponent = true;
            cursor += 1;
            exponent_negative := cursor < raw_count && raw[cursor] == 45;

            if cursor < raw_count && (raw[cursor] == 45 || raw[cursor] == 43) cursor += 1;

            while cursor < raw_count {
                exponent = exponent * 10 + raw[cursor] - 48;
                cursor += 1;
            }

            if exponent_negative exponent = -exponent;

            break;
        } else if digit_count < digits.count {
            digits[digit_count] = c;
            digit_count += 1;
        }

        cursor += 1;
    }

    if point < 0 point = digit_count;

    // `0.05` arrives as digits 005 with the point after the first.
    while digit_count > 1 && digits[0] == 48 {
        for 1..digit_count - 1 digits[it - 1] = digits[it];

        digit_count -= 1;
        point -= 1;
    }

    // Position of the decimal point relative to the digit string.
    point += exponent;

    out: [64] u8 = ---;
    count := 0;

    if negative {
        out[0] = 45;
        count = 1;
    }

    if has_exponent && (point > 21 || point < -4) {
        // Scientific: d[.ddd]e+XX, shortest digits as printed.
        out[count] = digits[0];
        count += 1;

        if digit_count > 1 {
            out[count] = 46;
            count += 1;

            for 1..digit_count - 1 {
                out[count] = digits[it];
                count += 1;
            }
        }

        out[count] = 101;
        count += 1;
        scientific := point - 1;

        if scientific < 0 {
            out[count] = 45;
            scientific = -scientific;
        } else out[count] = 43;

        count += 1;

        if scientific >= 100 {
            out[count] = cast(u8) (48 + scientific / 100);
            count += 1;
        }

        out[count] = cast(u8) (48 + (scientific / 10) % 10);
        out[count + 1] = cast(u8) (48 + scientific % 10);
        count += 2;
    } else if point <= 0 {
        out[count] = 48;
        out[count + 1] = 46;
        count += 2;

        for 1..-point {
            out[count] = 48;
            count += 1;
        }

        for 0..digit_count - 1 {
            out[count] = digits[it];
            count += 1;
        }
    } else {
        for 0..max(point, digit_count) - 1 {
            if it == point {
                out[count] = 46;
                count += 1;
            }

            out[count] = ifx it < digit_count then digits[it] else 48;
            count += 1;
        }

        if point >= digit_count {
            out[count] = 46;
            out[count + 1] = 48;
            count += 2;
        }
    }

    toml_emit(w, string.{ count, out.data });
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

// Duplicate detection for a table without a trusted index: an open-addressing set of member
// numbers with a 32-bit tag per slot. Keys are hashed with a seed picked per write, so keys an
// attacker chooses cannot be steered into one bucket; the structure never outlives the write.
Toml_Key_Slot :: struct {
    tag: u32;
    member: u32; // member number plus one; zero is empty
}

Toml_Key_Set :: struct {
    slots: [] Toml_Key_Slot;
    mask: u64;
    seed: u64;
    inline_slots: [256] Toml_Key_Slot = ---;
    heap: bool;
    usable: bool;
}

toml_hash_key :: (key: string, seed: u64) -> u64 {
    hash := seed ^ (cast(u64) key.count * 0x9e3779b97f4a7c15);
    at := 0;

    while at + 8 <= key.count {
        word: u64 = ---;

        memcpy(*word, key.data + at, 8);
        hash = (hash ^ word) * 0xff51afd7ed558ccd;
        hash ^= hash >> 32;
        at += 8;
    }

    if at < key.count {
        word: u64 = 0;

        memcpy(*word, key.data + at, key.count - at);
        hash = (hash ^ word) * 0xc4ceb9fe1a85ec53;
        hash ^= hash >> 29;
    }

    hash ^= hash >> 33;
    hash *= 0xff51afd7ed558ccd;
    hash ^= hash >> 33;

    return hash;
}

// `expected` is the number of keys that will be added at most.
toml_key_set_init :: (set: *Toml_Key_Set, w: *Toml_Writer, expected: int) {
    if expected <= 0 || expected >= 0x40000000 return;

    capacity := 32;
    while capacity < expected * 2 capacity *= 2;

    if capacity <= set.inline_slots.count {
        set.slots = .{ capacity, set.inline_slots.data };
    } else {
        bytes := capacity * size_of(Toml_Key_Slot);

        set.slots = .{ capacity, cast(*Toml_Key_Slot) alloc(bytes) };
        set.heap = true;
    }

    memset(set.slots.data, 0, set.slots.count * size_of(Toml_Key_Slot));

    set.mask = cast(u64) set.slots.count - 1;
    set.seed = w.seed;
    set.usable = true;
}

toml_key_set_free :: (set: *Toml_Key_Set) {
    if set.heap free(set.slots.data);
}

// False when the key of member `index` equals an earlier member's key.
toml_key_set_add :: (set: *Toml_Key_Set, members: [] Toml_Member, index: int) -> bool {
    key := members[index].key;

    // Without a set (an absurd member count, or no node quota left) compare directly.
    if !set.usable {
        for previous: 0..index - 1 if members[previous].key == key return false;

        return true;
    }

    hash := toml_hash_key(key, set.seed);
    tag := cast(u32) (hash >> 32);
    slot := hash & set.mask;

    while true {
        entry := *set.slots[slot];

        if !entry.member {
            entry.tag = tag;
            entry.member = cast(u32) index + 1;

            return true;
        }

        if entry.tag == tag && members[entry.member - 1].key == key return false;

        slot = (slot + 1) & set.mask;
    }

    return true;
}

// A table written under its own `[header]`, empty or not.
toml_is_header_table :: (node: *Toml_Value) -> bool {
    return node && node.kind == .TABLE;
}

// A nonempty array whose items are all tables, written as `[[header]]` per item.
toml_is_table_array :: (node: *Toml_Value) -> bool {
    if !node || node.kind != .ARRAY || !node.items.count return false;

    for node.items if !it || it.kind != .TABLE return false;

    return true;
}

// Writes the members of `node`. With `inline_table` they are `{a = 1, b = 2}`; otherwise they
// are the body of the table named by `w.path`: `key = value` lines first, then the headers of
// its tables and arrays of tables in member order (TOML needs that order).
toml_write_table :: (w: *Toml_Writer, node: *Toml_Value, depth: int, inline_table: bool) {
    // A cached index that proves uniqueness (parsed and builder tables) needs no key set.
    // Do not pre-walk more immediate members than the remaining node quota permits.
    // Incremental checks preserve the existing first-error path for smaller limits.
    proven := node.members.count <= w.options.max_nodes - w.nodes && toml_index_proves_unique(node);
    tiny := node.members.count <= 16;
    keys: Toml_Key_Set;

    if !proven && !tiny toml_key_set_init(*keys, w, min(node.members.count, w.options.max_nodes - w.nodes + 1));

    defer toml_key_set_free(*keys);
    if inline_table toml_emit(w, "{");

    for node.members {
        saved_path := w.path.count;

        array_add(*w.path, Toml_Path_Part.{ name = it.key });

        if !proven {
            duplicate := -1;

            if tiny {
                for previous: 0..it_index - 1 {
                    if node.members[previous].key == it.key {
                        duplicate = previous;
                        break;
                    }
                }
            } else if !toml_key_set_add(*keys, node.members, it_index) duplicate = 0;

            if duplicate >= 0 {
                toml_write_fail(w, .OUTPUT, "duplicate constructed table key");
                return;
            }
        }

        if !inline_table && (toml_is_header_table(it.value) || toml_is_table_array(it.value)) {
            w.path.count = saved_path;
            continue;
        }

        if inline_table && it_index toml_emit(w, ", ");

        toml_write_key(w, it.key, " = ");
        toml_write_value(w, it.value, depth + 1);
        w.path.count = saved_path;
        if !inline_table toml_emit(w, "\n");
        if w.error.kind != .NONE return;
    }

    if inline_table {
        toml_emit(w, "}");
        return;
    }

    for node.members {
        is_table := toml_is_header_table(it.value);

        if !is_table && !toml_is_table_array(it.value) continue;

        saved_path := w.path.count;

        array_add(*w.path, Toml_Path_Part.{ name = it.key });

        if !toml_count_node(w, it.value, depth + 1) return;

        if is_table {
            toml_write_header(w, false);
            toml_write_table(w, it.value, depth + 1, false);
        } else {
            for item: it.value.items {
                item_path := w.path.count;

                array_add(*w.path, Toml_Path_Part.{ index = it_index });
                if !toml_count_node(w, item, depth + 2) return;

                toml_write_header(w, true);
                toml_write_table(w, item, depth + 2, false);
                w.path.count = item_path;
                if w.error.kind != .NONE return;
            }
        }

        w.path.count = saved_path;
        if w.error.kind != .NONE return;
    }
}

// ---- typed output ----

toml_encode_text :: (value: *$T, options: Toml_Options) -> string, Toml_Error {
    writer: Toml_Writer;
    writer.options = options;

    defer free_buffers(*writer.builder);
    defer free_buffers(*writer.float_scratch);
    defer array_free(writer.path);

    if !toml_options_valid(options) {
        toml_write_fail(*writer, .LIMIT, "limits must be nonnegative");
        return "", writer.error;
    }

    toml_encode_body(*writer, value);

    if writer.error.kind != .NONE return "", writer.error;
    if writer.builder.failed return "", .{ kind = .OUTPUT, message = "output allocation failed" };

    return builder_to_string(*writer.builder), .{};
}

// Counts one table or array against the node and depth limits; pair with toml_end_value.
toml_begin_value :: (w: *Toml_Writer) -> bool {
    if w.error.kind != .NONE return false;

    w.nodes += 1;

    if w.nodes > w.options.max_nodes {
        toml_write_fail(w, .LIMIT, "writer node limit exceeded");
        return false;
    }

    w.depth += 1;

    if w.depth > w.options.max_depth {
        toml_write_fail(w, .LIMIT, "writer nesting limit exceeded");
        return false;
    }

    return true;
}

toml_end_value :: (w: *Toml_Writer) {
    w.depth -= 1;
}

toml_path_index_for_write :: (w: *Toml_Writer, index: int) -> int {
    saved := w.path.count;
    array_add(*w.path, Toml_Path_Part.{ index = index });

    return saved;
}

// `[a.b]` or `[[a.b]]` for the table named by the path (array positions are not part of it),
// after a blank line unless it opens the output.
toml_write_header :: (w: *Toml_Writer, array: bool) {
    if w.bytes > 0 toml_emit(w, "\n");

    toml_emit(w, ifx array then "[[" else "[");

    first := true;

    for w.path if it.index < 0 {
        if !first toml_emit(w, ".");

        first = false;
        toml_write_key(w, it.name);
    }

    toml_emit(w, ifx array then "]]\n" else "]\n");
}

// TOML integers are signed 64-bit.
toml_write_unsigned :: (w: *Toml_Writer, value: u64) {
    if value > 0x7fffffffffffffff {
        toml_write_fail(w, .RANGE, "integer out of signed 64-bit range");
        return;
    }

    toml_write_integer(w, cast(s64) value);
}

toml_write_datetime :: (w: *Toml_Writer, value: *Toml_Datetime) {
    node: Toml_Value;

    node.kind = value.kind;
    node.temporal = value.value;
    toml_write_value(w, *node, 0);
}

toml_write_fail :: (w: *Toml_Writer, kind: Toml_Error_Kind, message: string) {
    if w.error.kind != .NONE return;

    w.error.kind = kind;
    w.error.message = message;
    reader: Toml_Reader;
    reader.path = w.path;
    toml_render_path(*reader, *w.error);
}
