#scope_module
Toml_Declaration_Kind :: enum u8 {
    IMPLICIT;
    EXPLICIT;
    DOTTED;
    CLOSED;
    ARRAY_TABLE;
}

Toml_Declaration :: struct {
    kind: Toml_Declaration_Kind;
    span: Toml_Span;
    depth: int;
}

Toml_Path_Part :: struct {
    name: string;
    index: int = -1;
    owned: bool;
}

Toml_Key_Buffer :: struct {
    values: [..] string;
}

Toml_Reader :: struct {
    text: string;
    pos: int;
    options: Toml_Options;
    doc: Toml_Document;
    declarations: [..] Toml_Declaration;
    error: Toml_Error;
    depth: int;
    header_depth: int;
    line_scan: int;
    statement_line: int = 1;
    path: [..] Toml_Path_Part;
    key_buffers: [..] Toml_Key_Buffer;
    key_scope: int;

    // Typed decoding (decode.jai) reuses the scalar and string readers without a document:
    // each scalar lands in `scratch` and is consumed at once.
    stream: bool;
    scratch: Toml_Value;
    stream_nodes: int;
    statement_at: int;
    value_at: int;
    header_keys: [..] string;
}

toml_fail :: (r: *Toml_Reader, kind: Toml_Error_Kind, message: string, at: int = -1, related := Toml_Span.{}) -> bool {
    if r.error.kind != .NONE return false;
    if at < 0 at = r.pos;

    at = max(0, min(at, r.text.count));
    r.error.kind = kind;
    r.error.message = message;
    r.error.span = .{ at, min(at + 1, r.text.count) };
    r.error.related = related;
    toml_render_path(r, *r.error);
    r.error.line = 1;
    r.error.column = 1;

    for 0..at - 1 {
        c := r.text[it];
        if c == 10 {
            r.error.line += 1;
            r.error.column = 1;
        } else if (c & 0xc0) != 0x80 r.error.column += 1;
    }

    return false;
}

// The explicit remaining-length check proves the pointer load, including at truncated input.
// This tiny per-byte operation must not copy a call context or call a second indexing helper.
toml_peek :: inline (r: *Toml_Reader, ahead: int = 0) -> u8 {
    if r.pos < 0 || ahead < 0 || r.pos >= r.text.count || ahead >= r.text.count - r.pos return 0;
    return r.text.data[r.pos + ahead];
}

toml_space :: (r: *Toml_Reader) {
    while toml_peek(r) == 32 || toml_peek(r) == 9 r.pos += 1;
}

toml_newline :: (r: *Toml_Reader) -> bool {
    if toml_peek(r) == 13 {
        if toml_peek(r, 1) != 10 return toml_fail(r, .SYNTAX, "CR must be followed by LF");
        r.pos += 1;
    }

    if toml_peek(r) == 10 {
        r.pos += 1;
        return true;
    }

    return false;
}

toml_gap :: (r: *Toml_Reader) -> bool {
    while true {
        toml_space(r);

        if toml_peek(r) == 35 {
            while r.pos < r.text.count && toml_peek(r) != 10 && toml_peek(r) != 13 {
                c := toml_peek(r);
                if (c < 32 && c != 9) || c == 127 return toml_fail(r, .SYNTAX, "control character in comment");
                r.pos += 1;
            }
        }

        if !toml_newline(r) break;
    }

    return r.error.kind == .NONE;
}

toml_node :: (r: *Toml_Reader, kind: Toml_Kind) -> *Toml_Value {
    if r.stream {
        r.stream_nodes += 1;

        if r.stream_nodes > r.options.max_nodes {
            toml_fail(r, .LIMIT, "node limit exceeded");
            return null;
        }

        r.scratch = .{};
        r.scratch.kind = kind;

        return *r.scratch;
    }

    if r.doc.nodes.count >= r.options.max_nodes {
        toml_fail(r, .LIMIT, "node limit exceeded");
        return null;
    }

    node_growth, _ := toml_array_growth(*r.doc.nodes);
    declaration_growth, _ := toml_array_growth(*r.declarations);

    if !toml_admit(*r.doc, node_growth, declaration_growth, toml_node_storage_growth(*r.doc)) {
        toml_fail(r, .LIMIT, "retained byte limit exceeded");
        return null;
    }

    toml_reserve(r, *r.doc.nodes);
    toml_reserve(r, *r.declarations);
    node := toml_allocate_node(*r.doc);
    node.kind = kind;
    if kind == .LOCAL_DATE || kind == .LOCAL_TIME || kind == .LOCAL_DATETIME || kind == .OFFSET_DATETIME node.temporal = .{};

    node.internal_index = r.doc.nodes.count;
    array_add(*r.doc.nodes, node);
    array_add(*r.declarations, Toml_Declaration.{ depth = r.depth });

    return node;
}

toml_duplicate :: (r: *Toml_Reader, value: *Toml_Value, at: int) -> bool {
    return toml_fail(r, .DUPLICATE, "key or table is already defined", at, r.declarations[value.internal_index].span);
}

toml_bare :: (c: u8) -> bool {
    return (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 45 || c == 95;
}

toml_keys :: (r: *Toml_Reader, keys: *[..] string) -> bool {
    while true {
        toml_space(r);
        if !toml_reserve(r, keys) return false;

        part: string;

        if toml_peek(r) == 34 || toml_peek(r) == 39 {
            if !toml_string(r, *part, true) return false;
        } else {
            start := r.pos;

            while toml_bare(toml_peek(r)) r.pos += 1;
            if start == r.pos return toml_fail(r, .SYNTAX, "expected a key");

            length := r.pos - start;
            if length > r.options.max_string_bytes return toml_fail(r, .LIMIT, "decoded key byte limit exceeded");

            part = slice(r.doc.source, start, length);
        }

        array_add(keys, part);
        if keys.count > r.options.max_depth return toml_fail(r, .LIMIT, "key depth limit exceeded");

        toml_space(r);
        if toml_peek(r) != 46 return true;

        r.pos += 1;
    }

    return false;
}

// Active scopes own their backing arrays locally: recursive parsing may grow key_buffers,
// so only an integer slot survives calls. Cached capacities remain in the retained budget.
toml_acquire_keys :: (r: *Toml_Reader, keys: *[..] string) -> int {
    slot := r.key_scope;

    if slot == r.key_buffers.count {
        if !toml_reserve(r, *r.key_buffers) return -1;
        array_add(*r.key_buffers, Toml_Key_Buffer.{});
    }

    keys.* = r.key_buffers[slot].values;
    r.key_buffers[slot].values = .{};
    r.key_scope += 1;

    return slot;
}

toml_release_keys :: (r: *Toml_Reader, keys: *[..] string, slot: int) {
    for keys.* {
        if !toml_key_uses_source(r.doc.source, it) {
            if it.data free(it);
            r.doc.retained_bytes -= it.count;
        }
    }

    keys.count = 0;
    assert(slot == r.key_scope - 1);
    r.key_buffers[slot].values = keys.*;
    keys.* = .{};
    r.key_scope -= 1;
}

// Parsed keys use the retained source or an owned, charged decoded copy. Transfer only after capacity admission;
// on failure the caller still owns *key and its deferred cleanup remains valid.
toml_take_member :: (r: *Toml_Reader, table: *Toml_Value, key: *string, value: *Toml_Value, span: Toml_Span, hint: *Toml_Insert_Hint) {
    depth := r.declarations[table.internal_index].depth;
    if depth >= r.options.max_depth {
        toml_fail(r, .LIMIT, "table depth limit exceeded");
        return;
    }

    duplicate := toml_prepare_insertion(table, table.internal_lookup, key.*, hint);
    if duplicate >= 0 {
        toml_duplicate(r, table.members[duplicate].value, span.start);
        return;
    }

    growth, _ := toml_array_growth(*table.members);
    if !toml_admit(*r.doc, growth, toml_index_growth(table)) {
        toml_fail(r, .LIMIT, "retained byte limit exceeded");
        return;
    }

    toml_reserve(r, *table.members);
    toml_prepare_index(*r.doc, table);
    array_add(*table.members, Toml_Member.{ key.*, value, span });
    key.* = .{};

    if table.internal_lookup {
        duplicate = toml_tree_add(table, table.internal_lookup, table.members.count - 1, hint, true);
        if duplicate >= 0 {
            toml_duplicate(r, table.members[duplicate].value, span.start);
            return;
        }
    }

    r.declarations[value.internal_index].span = span;
    r.declarations[value.internal_index].depth = depth + 1;
    toml_note_child(table, value, table.members.count - 1);
}

// Walk declaration paths with distinct rules for dotted assignments and table headers.
toml_parent :: (r: *Toml_Reader, start: *Toml_Value, keys: [] string, header: bool, at: int) -> *Toml_Value {
    table := start;

    for keys {
        hint: Toml_Insert_Hint;
        found := toml_probe_index(table, table.internal_lookup, it, *hint);
        node := ifx found >= 0 then table.members[found].value else null;

        if !node {
            node = toml_node(r, .TABLE);
            if !node return null;

            node.span = .{ at, r.pos };
            r.declarations[node.internal_index].kind = ifx header then .IMPLICIT else .DOTTED;
            toml_take_member(r, table, keys.data + it_index, node, .{ at, r.pos }, *hint);
        } else {
            declaration := r.declarations[node.internal_index].kind;
            if !header && declaration == .IMPLICIT {
                r.declarations[node.internal_index].kind = .DOTTED;
                r.declarations[node.internal_index].span = .{ at, r.pos };
            }

            if header && declaration == .ARRAY_TABLE {
                node = node.items[node.items.count - 1];
            } else if node.kind != .TABLE || declaration == .CLOSED || (!header && declaration == .EXPLICIT) {
                toml_duplicate(r, node, at);
                return null;
            }
        }

        table = node;
    }

    if r.error.kind != .NONE return null;

    return table;
}

toml_pair :: (r: *Toml_Reader, table: *Toml_Value) -> bool {
    if r.error.kind != .NONE return false;

    at := r.pos;
    keys: [..] string;

    key_slot := toml_acquire_keys(r, *keys);

    if key_slot < 0 return false;

    defer toml_release_keys(r, *keys, key_slot);
    if !toml_keys(r, *keys) return false;
    if toml_peek(r) != 61 return toml_fail(r, .SYNTAX, "expected '=' after key");

    r.pos += 1;
    toml_space(r);
    path_count := r.path.count;

    defer r.path.count = path_count;

    for keys {
        if !toml_reserve(r, *r.path) return false;
        array_add(*r.path, Toml_Path_Part.{ name = it });
    }

    parent := toml_parent(r, table, ([] string).{ keys.count - 1, keys.data }, false, at);
    if !parent return false;

    saved_depth := r.depth;

    r.depth = r.declarations[parent.internal_index].depth;
    defer r.depth = saved_depth;
    key := keys[keys.count - 1];
    hint: Toml_Insert_Hint;
    found := toml_probe_index(parent, parent.internal_lookup, key, *hint);
    existing := ifx found >= 0 then parent.members[found].value else null;

    if existing return toml_duplicate(r, existing, at);

    statement := toml_begin_statement(r, .ASSIGNMENT, at, r.header_depth);
    if statement == -2 return false;

    value := toml_value(r);
    if !value return false;

    toml_take_member(r, parent, keys.data + keys.count - 1, value, .{ at, value.span.start }, *hint);
    // Related diagnostics should point to the original key, not only its value.
    r.declarations[value.internal_index].span = .{ at, value.span.end };

    if statement >= 0 {
        r.doc.statements[statement].value = value;
        r.doc.statements[statement].span.end = value.span.end;
    }

    return r.error.kind == .NONE;
}

// Parse one header, retaining declaration/source history before selecting its table.
toml_header :: (r: *Toml_Reader) -> *Toml_Value {
    current: *Toml_Value;
    at := r.pos;

    r.pos += 1;
    array_table := toml_peek(r) == 91;
    if array_table r.pos += 1;

    keys: [..] string;

    key_slot := toml_acquire_keys(r, *keys);

    if key_slot < 0 return null;

    defer toml_release_keys(r, *keys, key_slot);
    if !toml_keys(r, *keys) return null;

    if toml_peek(r) != 93 {
        toml_fail(r, .SYNTAX, "expected ']' after table name");
        return null;
    }

    r.pos += 1;

    if array_table {
        if toml_peek(r) != 93 {
            toml_fail(r, .SYNTAX, "expected ']]' after array table name");
            return null;
        }

        r.pos += 1;
    }

    if !toml_header_path(r, keys, array_table) return null;

    parent := toml_parent(r, r.doc.root, ([] string).{ keys.count - 1, keys.data }, true, at);
    if !parent return null;

    key := keys[keys.count - 1];
    hint: Toml_Insert_Hint;
    found := toml_probe_index(parent, parent.internal_lookup, key, *hint);
    node := ifx found >= 0 then parent.members[found].value else null;

    if array_table {
        if node && r.declarations[node.internal_index].kind != .ARRAY_TABLE {
            toml_duplicate(r, node, at);
            return null;
        }

        if !node {
            node = toml_node(r, .ARRAY);
            if !node return null;

            node.span = .{ at, r.pos };
            r.declarations[node.internal_index].kind = .ARRAY_TABLE;
            toml_take_member(r, parent, keys.data + keys.count - 1, node, .{ at, r.pos }, *hint);
        }

        current = toml_node(r, .TABLE);
        if !current return null;

        current.span = .{ at, r.pos };
        r.declarations[current.internal_index].depth = r.declarations[node.internal_index].depth + 1;
        if r.declarations[current.internal_index].depth > r.options.max_depth {
            toml_fail(r, .LIMIT, "table depth limit exceeded");
            return null;
        }

        r.declarations[current.internal_index].kind = .EXPLICIT;
        if !toml_reserve(r, *node.items) return null;

        array_add(*node.items, current);
        toml_note_child(node, current, node.items.count - 1);
    } else {
        if node {
            if node.kind != .TABLE || r.declarations[node.internal_index].kind != .IMPLICIT {
                toml_duplicate(r, node, at);
                return null;
            }
        } else {
            node = toml_node(r, .TABLE);
            if !node return null;
            toml_take_member(r, parent, keys.data + keys.count - 1, node, .{ at, r.pos }, *hint);
        }

        node.span = .{ at, r.pos };
        r.declarations[node.internal_index].kind = .EXPLICIT;
        r.declarations[node.internal_index].span = .{ at, r.pos };
        current = node;
    }

    r.header_depth = keys.count;
    statement := toml_begin_statement(r, ifx array_table then .ARRAY_TABLE else .TABLE, at, keys.count);
    if statement == -2 return null;

    if statement >= 0 {
        r.doc.statements[statement].value = current;
        r.doc.statements[statement].span.end = r.pos;
    }

    return current;
}

toml_document :: (r: *Toml_Reader) -> bool {
    current := r.doc.root;

    while toml_gap(r) && r.pos < r.text.count {
        if toml_peek(r) == 91 {
            current = toml_header(r);
            if !current return false;
        } else if !toml_pair(r, current) return false;

        toml_space(r);

        if toml_peek(r) == 35 {
            while r.pos < r.text.count && toml_peek(r) != 10 && toml_peek(r) != 13 {
                c := toml_peek(r);
                if (c < 32 && c != 9) || c == 127 return toml_fail(r, .SYNTAX, "control character in comment");
                r.pos += 1;
            }
        }

        if r.pos < r.text.count && !toml_newline(r) return toml_fail(r, .SYNTAX, "expected newline after statement");
    }

    return r.error.kind == .NONE;
}

toml_value :: (r: *Toml_Reader) -> *Toml_Value {
    if r.depth >= r.options.max_depth {
        toml_fail(r, .LIMIT, "value nesting limit exceeded");
        return null;
    }

    r.depth += 1;
    defer r.depth -= 1;
    start := r.pos;
    c := toml_peek(r);
    node: *Toml_Value;

    if c == 34 || c == 39 {
        node = toml_node(r, .STRING);
        if !node || !toml_string(r, *node.text) return null;
    } else if c == 91 {
        r.pos += 1;
        node = toml_node(r, .ARRAY);
        if !node || !toml_gap(r) return null;

        if toml_peek(r) != 93 {
            while true {
                saved_path := r.path.count;

                if !toml_reserve(r, *r.path) return null;

                array_add(*r.path, Toml_Path_Part.{ index = node.items.count });
                item := toml_value(r);

                r.path.count = saved_path;
                if !item return null;
                if !toml_reserve(r, *node.items) return null;

                array_add(*node.items, item);
                toml_note_child(node, item, node.items.count - 1);
                if !toml_gap(r) return null;
                if toml_peek(r) == 93 break;

                if toml_peek(r) != 44 {
                    toml_fail(r, .SYNTAX, "expected ',' or ']' in array");
                    return null;
                }

                r.pos += 1;
                if !toml_gap(r) return null;
                if toml_peek(r) == 93 break;
            }
        }

        r.pos += 1;
    } else if c == 123 {
        r.pos += 1;
        node = toml_node(r, .TABLE);
        if !node || !toml_gap(r) return null;

        if toml_peek(r) != 125 {
            while true {
                if !toml_pair(r, node) || !toml_gap(r) return null;
                if toml_peek(r) == 125 break;

                if toml_peek(r) != 44 {
                    toml_fail(r, .SYNTAX, "expected ',' or '}' in inline table");
                    return null;
                }

                r.pos += 1;
                if !toml_gap(r) return null;
                if toml_peek(r) == 125 break;
            }
        }

        r.pos += 1;
        r.declarations[node.internal_index].kind = .CLOSED;
    } else {
        node = toml_scalar(r, toml_token(r), start);
        if !node return null;
    }

    node.span = .{ start, r.pos };

    return node;
}

// The bytes of one scalar token, which ends at a delimiter or comment.
toml_token :: (r: *Toml_Reader) -> string {
    start := r.pos;

    while r.pos < r.text.count {
        byte := toml_peek(r);
        if byte == 9 || byte == 10 || byte == 13 || byte == 35 || byte == 44 || byte == 93 || byte == 125 break;

        if byte == 32 {
            // A date and time can be separated by one space.
            if r.pos - start != 10 || toml_peek(r, 1) < 48 || toml_peek(r, 1) > 57 break;
        }

        r.pos += 1;
    }

    return slice(r.text, start, r.pos - start);
}

// Account capacity before every retained allocation; allocator metadata is outside this limit.
toml_reserve :: (r: *Toml_Reader, array: *[..] $T) -> bool {
    if !toml_build_reserve(*r.doc, array) return toml_fail(r, .LIMIT, "retained byte limit exceeded");
    return true;
}

toml_clear_path :: (r: *Toml_Reader) {
    for r.path if it.owned {
        if it.name.data free(it.name);
        r.doc.retained_bytes -= it.name.count;
    }

    r.path.count = 0;
}

toml_header_path :: (r: *Toml_Reader, keys: [] string, array_table: bool) -> bool {
    toml_clear_path(r);
    table := r.doc.root;

    for keys {
        growth, _ := toml_array_growth(*r.path);
        if !toml_admit(*r.doc, growth, it.count) return toml_fail(r, .LIMIT, "retained byte limit exceeded");

        toml_reserve(r, *r.path);
        array_add(*r.path, Toml_Path_Part.{ name = copy_string(it), owned = true });
        r.doc.retained_bytes += it.count;
        node := toml_get(table, it);
        if node && r.declarations[node.internal_index].kind == .ARRAY_TABLE {
            if !toml_reserve(r, *r.path) return false;
            last := array_table && it_index == keys.count - 1;
            array_add(*r.path, Toml_Path_Part.{ index = ifx last then node.items.count else node.items.count - 1 });
            table = node.items[node.items.count - 1];
        } else {
            if array_table && it_index == keys.count - 1 && !node {
                if !toml_reserve(r, *r.path) return false;
                array_add(*r.path, Toml_Path_Part.{ index = 0 });
            }

            table = node;
        }
    }

    return true;
}

// Render whole segments only, so a truncated path is still valid UTF-8 and never borrows input.
toml_render_path :: (r: *Toml_Reader, error: *Toml_Error) {
    error.path_bytes[0] = 36;
    error.path_count = 1;

    for part: r.path {
        if part.index >= 0 {
            digits: [32] u8;
            count := 0;
            n := part.index;

            while true {
                digits[count] = cast(u8) (48 + n % 10);
                count += 1;
                n /= 10;
                if !n break;
            }

            if count + 2 > error.path_bytes.count - error.path_count {
                error.path_truncated = true;
                return;
            }

            error.path_bytes[error.path_count] = 91;
            error.path_count += 1;

            for j: 0..count - 1 {
                error.path_bytes[error.path_count] = digits[count - 1 - j];
                error.path_count += 1;
            }

            error.path_bytes[error.path_count] = 93;
            error.path_count += 1;
        } else {
            length := 4;
            for byte: part.name length += ifx byte < 32 || byte == 127 then 6 else ifx byte == 34 || byte == 92 then 2 else 1;

            if length > error.path_bytes.count - error.path_count {
                error.path_truncated = true;
                return;
            }

            error.path_bytes[error.path_count] = 91;
            error.path_count += 1;
            error.path_bytes[error.path_count] = 34;
            error.path_count += 1;

            for byte: part.name {
                if byte < 32 || byte == 127 {
                    hex := "0123456789abcdef";
                    escape := u8.[92, 117, 48, 48, hex[byte >> 4], hex[byte & 15]];

                    for escape {
                        error.path_bytes[error.path_count] = it;
                        error.path_count += 1;
                    }
                } else {
                    if byte == 34 || byte == 92 {
                        error.path_bytes[error.path_count] = 92;
                        error.path_count += 1;
                    }

                    error.path_bytes[error.path_count] = byte;
                    error.path_count += 1;
                }
            }

            error.path_bytes[error.path_count] = 34;
            error.path_count += 1;
            error.path_bytes[error.path_count] = 93;
            error.path_count += 1;
        }
    }
}

toml_free_scratch :: (r: *Toml_Reader) {
    assert(r.key_scope == 0);

    for *r.key_buffers {
        r.doc.retained_bytes -= it.values.allocated * size_of(string);
        array_reset(*it.values);
    }

    r.doc.retained_bytes -= r.key_buffers.allocated * size_of(Toml_Key_Buffer);
    array_reset(*r.key_buffers);
    toml_clear_path(r);
    r.doc.retained_bytes -= r.path.allocated * size_of(Toml_Path_Part);
    r.doc.retained_bytes -= r.declarations.allocated * size_of(Toml_Declaration);
    array_reset(*r.path);
    array_reset(*r.declarations);
}
