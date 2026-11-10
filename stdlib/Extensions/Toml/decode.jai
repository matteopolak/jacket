// Run-time half of typed decoding. The reader, the string and number grammar are the document
// parser's; this file replaces the document with a tree of declaration records: one small
// frame per table the input mentions and one record per key, with no values. That is all
// TOML's rules about duplicate keys and table redefinition need. Values go straight into the
// target; generate.jai writes the per-type code that looks keys up and stores them.
#scope_module

// How one key (or table header segment) was introduced. A value's own record is `declared` with
// kind CLOSED: assigned values, including arrays and inline tables, can never be extended.
Toml_Decl :: struct {
    declared: bool;
    kind: Toml_Declaration_Kind;
    span: Toml_Span;
    child: *Toml_Frame;
    capacity: int; // of a `[]` view that `[[x]]` headers fill; its elements are appended in place
}

Toml_Unknown_State :: struct {
    name: string;
    owned: bool;
    elements: int; // `[[name]]` headers seen so far
    member: int; // in a `Toml_Value` table: the position of this key's member
    decl: Toml_Decl;
}

// The declarations inside one table. `fields` has one record per field of the target type; keys the
// type does not know are tracked by name in `unknown`, so even ignored data obeys TOML's rules.
// A frame with many unknown keys also keeps `unknown_index`, an open-addressing table of
// positions in `unknown` plus one (zero is empty), built once the linear scan would get long.
Toml_Frame :: struct {
    fields: [..] Toml_Decl;
    unknown: [..] Toml_Unknown_State;
    unknown_index: [..] u32;
}

Toml_Op :: enum u8 {
    HEADER;
    HEADER_ARRAY;
    ASSIGN;
}

// What one segment of a key path means: intermediate parts of a header (or the header an
// assignment sits under), intermediate parts of a dotted key, or the last segment.
Toml_Step :: enum u8 {
    MID_HEADER;
    MID_DOTTED;
    FINAL_TABLE;
    FINAL_ARRAY;
    FINAL_ASSIGN;
}

toml_new_frame :: () -> *Toml_Frame {
    return New(Toml_Frame);
}

toml_frame_fields :: (frame: *Toml_Frame, count: int) {
    if frame.fields.count < count array_resize(*frame.fields, count);
}

toml_free_frame :: (frame: *Toml_Frame) {
    if !frame return;

    for frame.fields if it.child toml_free_frame(it.child);

    for frame.unknown {
        if it.decl.child toml_free_frame(it.decl.child);
        if it.owned free(it.name.data);
    }

    array_free(frame.fields);
    array_free(frame.unknown);
    array_free(frame.unknown_index);
    free(frame);
}

// Frames with at most this many unknown keys are searched linearly.
TOML_UNKNOWN_SCAN :: 8;

// The frame's address varies per run, so keys chosen by an attacker cannot be steered into one bucket.
toml_unknown_hash :: (frame: *Toml_Frame, key: string) -> u64 {
    return toml_hash_key(key, cast(u64) cast(*void) frame);
}

toml_unknown_find :: (frame: *Toml_Frame, key: string) -> *Toml_Unknown_State {
    if !frame.unknown_index.count {
        for *frame.unknown if it.name == key return it;

        return null;
    }

    mask := cast(u64) frame.unknown_index.count - 1;
    slot := toml_unknown_hash(frame, key) & mask;

    while true {
        entry := frame.unknown_index[slot];

        if !entry return null;

        found := *frame.unknown[entry - 1];

        if found.name == key return found;

        slot = (slot + 1) & mask;
    }

    return null;
}

toml_unknown_index_insert :: (frame: *Toml_Frame, position: int) {
    mask := cast(u64) frame.unknown_index.count - 1;
    slot := toml_unknown_hash(frame, frame.unknown[position].name) & mask;

    while frame.unknown_index[slot] slot = (slot + 1) & mask;

    frame.unknown_index[slot] = cast(u32) position + 1;
}

// Indexes the entry just added to `unknown` (its name is set), building or growing the table
// when the scan has become long or the table is half full.
toml_unknown_register :: (frame: *Toml_Frame) {
    count := frame.unknown.count;

    if !frame.unknown_index.count && count <= TOML_UNKNOWN_SCAN return;

    if count * 2 <= frame.unknown_index.count {
        toml_unknown_index_insert(frame, count - 1);
        return;
    }

    capacity := 32;
    while capacity < count * 4 capacity *= 2;

    array_resize(*frame.unknown_index, capacity);
    memset(frame.unknown_index.data, 0, capacity * size_of(u32));

    for 0..count - 1 toml_unknown_index_insert(frame, it);
}

toml_step :: inline (op: Toml_Op, index: int, count: int, nav: int) -> Toml_Step {
    if index < count - 1 return ifx op != .ASSIGN || index < nav then Toml_Step.MID_HEADER else .MID_DOTTED;
    if op == .ASSIGN return .FINAL_ASSIGN;

    return ifx op == .HEADER then Toml_Step.FINAL_TABLE else .FINAL_ARRAY;
}

// The declaration rules, shared by known and unknown keys. A `[[x]]` header may repeat; the
// caller then starts a new element.
toml_enter :: (r: *Toml_Reader, decl: *Toml_Decl, step: Toml_Step, at: int) -> bool {
    span := Toml_Span.{ at, r.pos };

    if !decl.declared {
        if step != .FINAL_ASSIGN {
            r.stream_nodes += 1;
            if r.stream_nodes > r.options.max_nodes return toml_fail(r, .LIMIT, "node limit exceeded", at);
        }

        decl.declared = true;
        decl.span = span;

        if step == {
            case .MID_HEADER; decl.kind = .IMPLICIT;
            case .MID_DOTTED; decl.kind = .DOTTED;
            case .FINAL_TABLE; decl.kind = .EXPLICIT;
            case .FINAL_ARRAY; decl.kind = .ARRAY_TABLE;
            case; decl.kind = .CLOSED;
        }

        return true;
    }

    ok := false;

    if step == {
        case .MID_HEADER;
            ok = decl.kind != .CLOSED;
        case .MID_DOTTED;
            if decl.kind == .IMPLICIT {
                decl.kind = .DOTTED;
                decl.span = span;
                ok = true;
            } else {
                ok = decl.kind == .DOTTED;
            }
        case .FINAL_TABLE;
            if decl.kind == .IMPLICIT {
                decl.kind = .EXPLICIT;
                decl.span = span;
                ok = true;
            }
        case .FINAL_ARRAY;
            ok = decl.kind == .ARRAY_TABLE;
            if ok {
                r.stream_nodes += 1;
                if r.stream_nodes > r.options.max_nodes return toml_fail(r, .LIMIT, "node limit exceeded", at);
            }
        case; // Assigning to a declared key is always a duplicate.
    }

    if !ok return toml_fail(r, .DUPLICATE, "key or table is already defined", at, decl.span);

    return true;
}

// ---- values ----

// Reads one string, number, boolean or date into the scratch node. Arrays and tables are
// not scalars: `message` (or the default) reports them as the wrong type.
toml_read_leaf :: (r: *Toml_Reader, message: string, fallback: string) -> *Toml_Value {
    start := r.pos;
    c := toml_peek(r);

    if c == 91 || c == 123 {
        toml_fail(r, .TYPE, ifx message.count then message else fallback, start);
        return null;
    }

    node: *Toml_Value;

    if c == 34 || c == 39 {
        node = toml_node(r, .STRING);
        if !node || !toml_string(r, *node.text) return null;
    } else {
        node = toml_scalar(r, toml_token(r), start);
        if !node return null;
    }

    node.span = .{ start, r.pos };

    return node;
}

toml_read_string :: (r: *Toml_Reader, out: *string, message: string) -> bool {
    start := r.pos;
    c := toml_peek(r);

    if c != 34 && c != 39 return toml_fail(r, .TYPE, ifx message.count then message else "expected a string", start);

    node := toml_node(r, .STRING);
    if !node return false;

    text: string;
    if !toml_string(r, *text) return false;

    out.* = text;

    return true;
}

toml_read_bool :: (r: *Toml_Reader, out: *bool, message: string) -> bool {
    start := r.pos;
    node := toml_read_leaf(r, message, "expected true or false");

    if !node return false;

    if node.kind != .BOOL {
        toml_release_scratch(r, node);
        return toml_fail(r, .TYPE, ifx message.count then message else "expected true or false", start);
    }

    out.* = node.boolean;

    return true;
}

toml_read_signed :: (r: *Toml_Reader, out: *s64, low: s64, high: s64, message: string) -> bool {
    start := r.pos;
    node := toml_read_leaf(r, message, "expected an integer");

    if !node return false;

    if node.kind != .INTEGER {
        toml_release_scratch(r, node);
        return toml_fail(r, .TYPE, ifx message.count then message else "expected an integer", start);
    }

    if node.integer < low || node.integer > high return toml_fail(r, .RANGE, ifx message.count then message else "integer out of range", start);

    out.* = node.integer;

    return true;
}

toml_read_unsigned :: (r: *Toml_Reader, out: *u64, low: s64, high: s64, message: string) -> bool {
    start := r.pos;
    node := toml_read_leaf(r, message, "expected an integer");

    if !node return false;

    if node.kind != .INTEGER {
        toml_release_scratch(r, node);
        return toml_fail(r, .TYPE, ifx message.count then message else "expected an integer", start);
    }

    if node.integer < low || node.integer > high return toml_fail(r, .RANGE, ifx message.count then message else "integer out of range", start);

    out.* = cast(u64) node.integer;

    return true;
}

// Integers are accepted for floats, as in `x = 1`.
toml_read_float :: (r: *Toml_Reader, out: *float64, message: string) -> bool {
    start := r.pos;
    node := toml_read_leaf(r, message, "expected a float");

    if !node return false;

    if node.kind == .FLOAT out.* = node.real;
    else if node.kind == .INTEGER out.* = cast(float64) node.integer;
    else {
        toml_release_scratch(r, node);
        return toml_fail(r, .TYPE, ifx message.count then message else "expected a float", start);
    }

    return true;
}

toml_read_datetime :: (r: *Toml_Reader, out: *Toml_Datetime, message: string) -> bool {
    start := r.pos;
    node := toml_read_leaf(r, message, "expected a date or time");

    if !node return false;

    if node.kind != .LOCAL_DATE && node.kind != .LOCAL_TIME && node.kind != .LOCAL_DATETIME && node.kind != .OFFSET_DATETIME {
        toml_release_scratch(r, node);
        return toml_fail(r, .TYPE, ifx message.count then message else "expected a date or time", start);
    }

    out.kind = node.kind;
    out.value = node.temporal;

    return true;
}

// Frees what a scalar read allocated when the value is not kept.
toml_release_scratch :: (r: *Toml_Reader, node: *Toml_Value) {
    if node.kind == .STRING {
        if node.text.count {
            free(node.text.data);
            r.doc.retained_bytes -= node.text.count;
        }

        node.text = "";
    } else if node.kind == .LOCAL_DATE || node.kind == .LOCAL_TIME || node.kind == .LOCAL_DATETIME || node.kind == .OFFSET_DATETIME {
        if node.temporal.fraction.count {
            free(node.temporal.fraction.data);
            r.doc.retained_bytes -= node.temporal.fraction.count;
        }

        node.temporal.fraction = "";
    }
}

toml_enter_nested :: (r: *Toml_Reader, what: string) -> bool {
    if r.depth >= r.options.max_depth return toml_fail(r, .LIMIT, "value nesting limit exceeded");

    r.stream_nodes += 1;
    if r.stream_nodes > r.options.max_nodes return toml_fail(r, .LIMIT, "node limit exceeded");

    r.depth += 1;

    return true;
}

// Starts an array literal: false after recording an error.
toml_array_begin :: (r: *Toml_Reader, message: string) -> bool {
    if toml_peek(r) != 91 return toml_fail(r, .TYPE, ifx message.count then message else "expected an array");
    if !toml_enter_nested(r, "array") return false;

    r.pos += 1;

    return true;
}

// 1: another element follows, 0: the array ended, -1: error.
toml_array_next :: (r: *Toml_Reader, first: *bool) -> int {
    if !toml_gap(r) return -1;

    if toml_peek(r) == 93 {
        r.pos += 1;
        r.depth -= 1;
        return 0;
    }

    if !first.* {
        if toml_peek(r) != 44 {
            toml_fail(r, .SYNTAX, "expected ',' or ']' in array");
            return -1;
        }

        r.pos += 1;
        if !toml_gap(r) return -1;

        if toml_peek(r) == 93 {
            r.pos += 1;
            r.depth -= 1;
            return 0;
        }
    }

    first.* = false;

    return 1;
}

toml_path_index :: (r: *Toml_Reader, index: int) -> int {
    saved := r.path.count;
    array_add(*r.path, Toml_Path_Part.{ index = index });

    return saved;
}

// Validates and discards one value of any shape (data the target type has no field for).
toml_skip_value :: (r: *Toml_Reader) -> bool {
    c := toml_peek(r);

    if c == 91 {
        if !toml_array_begin(r, "") return false;

        first := true;
        index := 0;

        while true {
            status := toml_array_next(r, *first);
            if status < 0 return false;
            if status == 0 break;

            mark := toml_path_index(r, index);

            if !toml_skip_value(r) return false;

            r.path.count = mark;
            index += 1;
        }

        return true;
    }

    if c == 123 {
        if !toml_enter_nested(r, "table") return false;

        frame := toml_new_frame();

        defer toml_free_frame(frame);
        r.pos += 1;

        if !toml_gap(r) return false;

        if toml_peek(r) != 125 {
            while true {
                if !toml_unknown_pair(r, frame) || !toml_gap(r) return false;
                if toml_peek(r) == 125 break;

                if toml_peek(r) != 44 return toml_fail(r, .SYNTAX, "expected ',' or '}' in inline table");

                r.pos += 1;
                if !toml_gap(r) return false;
                if toml_peek(r) == 125 break;
            }
        }

        r.pos += 1;
        r.depth -= 1;

        return true;
    }

    node := toml_read_leaf(r, "", "");
    if !node return false;

    toml_release_scratch(r, node);

    return true;
}

toml_unknown_pair :: (r: *Toml_Reader, frame: *Toml_Frame) -> bool {
    keys: [..] string;
    slot := toml_acquire_keys(r, *keys);

    if slot < 0 return false;

    defer toml_release_keys(r, *keys, slot);
    saved_statement := r.statement_at;
    saved_value := r.value_at;

    defer r.statement_at = saved_statement;
    defer r.value_at = saved_value;
    r.statement_at = r.pos;
    if !toml_keys(r, *keys) return false;
    if toml_peek(r) != 61 return toml_fail(r, .SYNTAX, "expected '=' after key");

    r.pos += 1;
    toml_space(r);
    r.value_at = r.pos;

    saved := r.path.count;

    defer r.path.count = saved;
    array_add(*r.path, Toml_Path_Part.{ name = keys[0] });

    return toml_unknown_step(r, frame, keys, 0, .ASSIGN, 0);
}

// The record of `key` in a table whose keys are not fields of a struct: the keys of a
// `Hash_Table` or a `Toml_Value`, and the ignored keys of a struct. Added on first use.
toml_frame_entry :: (r: *Toml_Reader, frame: *Toml_Frame, key: string) -> *Toml_Unknown_State {
    entry := toml_unknown_find(frame, key);

    if !entry {
        entry = array_add(*frame.unknown);

        if toml_key_uses_source(r.doc.source, key) {
            entry.name = key;
        } else {
            entry.name = copy_string(key);
            entry.owned = true;
        }

        toml_unknown_register(frame);
    }

    return entry;
}

// Applies one segment of a key path to a key the target type has no field for, and everything
// below it. Nothing is stored: the records only decide whether the document is valid.
toml_unknown_step :: (r: *Toml_Reader, frame: *Toml_Frame, keys: [] string, index: int, op: Toml_Op, nav: int) -> bool {
    key := keys[index];
    entry := toml_frame_entry(r, frame, key);

    step := toml_step(op, index, keys.count, nav);

    if !toml_enter(r, *entry.decl, step, r.statement_at) return false;

    if step == .FINAL_ASSIGN return toml_skip_value(r);
    if step == .FINAL_TABLE return true;

    if step == .FINAL_ARRAY {
        toml_free_frame(entry.decl.child);
        entry.decl.child = toml_new_frame();
        entry.elements += 1;

        return true;
    }

    if !entry.decl.child entry.decl.child = toml_new_frame();

    // The caller of a step has put the key's name on the path; this one names the next key and,
    // below a `[[x]]` header, the element.
    if entry.decl.kind == .ARRAY_TABLE array_add(*r.path, Toml_Path_Part.{ index = entry.elements - 1 });

    array_add(*r.path, Toml_Path_Part.{ name = keys[index + 1] });

    return toml_unknown_step(r, entry.decl.child, keys, index + 1, op, nav);
}

// Notes one ignored key for a struct's `@toml(unknown)` list: its path below this table.
toml_record_unknown :: (r: *Toml_Reader, list: *[..] Toml_Unknown, keys: [] string, index: int, header: bool) {
    path: String_Builder;

    for index..keys.count - 1 {
        if it > index append(*path, ".");
        append(*path, keys[it]);
    }

    array_add(list, Toml_Unknown.{ path = builder_to_string(*path), start = r.statement_at, value = ifx header then -1 else r.value_at, end = r.pos, header = header });
}

toml_clear_header_keys :: (r: *Toml_Reader) {
    for r.header_keys {
        if it.data && !toml_key_uses_source(r.doc.source, it) {
            free(it.data);
            r.doc.retained_bytes -= it.count;
        }
    }

    r.header_keys.count = 0;
}

// ---- statements ----

// One `key = value` pair, either a statement of the document (`prefix` holds the keys of the
// header it sits under) or a member of an inline table (no prefix).
toml_decode_pair :: (r: *Toml_Reader, out: *$T, frame: *Toml_Frame, prefix: [] string) -> bool {
    if r.error.kind != .NONE return false;

    // An inline table's members are statements of their own; the enclosing one resumes after.
    saved_statement := r.statement_at;
    saved_value := r.value_at;

    defer r.statement_at = saved_statement;
    defer r.value_at = saved_value;
    r.statement_at = r.pos;

    keys: [..] string;
    slot := toml_acquire_keys(r, *keys);

    if slot < 0 return false;

    defer toml_release_keys(r, *keys, slot);
    if !toml_keys(r, *keys) return false;
    if toml_peek(r) != 61 return toml_fail(r, .SYNTAX, "expected '=' after key");

    r.pos += 1;
    toml_space(r);
    r.value_at = r.pos;

    saved_path := r.path.count;

    // The descent puts each key (and `[[x]]` element) on the path as it reaches it.
    defer r.path.count = saved_path;

    if !prefix.count return toml_descend(r, out, frame, keys, 0, .ASSIGN, 0);

    full: [..] string;
    full_slot := toml_acquire_keys(r, *full);

    if full_slot < 0 return false;

    defer toml_release_keys(r, *full, full_slot);

    // The released buffers free the keys they hold; these are copies of strings owned elsewhere.
    for prefix array_add(*full, it);
    for keys array_add(*full, it);

    result := toml_descend(r, out, frame, full, 0, .ASSIGN, prefix.count);

    // Do not let the release below free keys the header and pair still own.
    full.count = 0;

    return result;
}

toml_decode_header :: (r: *Toml_Reader, out: *$T, root: *Toml_Frame) -> bool {
    r.statement_at = r.pos;
    r.pos += 1;

    array_table := toml_peek(r) == 91;
    if array_table r.pos += 1;

    toml_clear_header_keys(r);
    r.path.count = 0;

    if !toml_keys(r, *r.header_keys) return false;

    if toml_peek(r) != 93 return toml_fail(r, .SYNTAX, "expected ']' after table name");

    r.pos += 1;

    if array_table {
        if toml_peek(r) != 93 return toml_fail(r, .SYNTAX, "expected ']]' after array table name");

        r.pos += 1;
    }

    return toml_descend(r, out, root, r.header_keys, 0, ifx array_table then Toml_Op.HEADER_ARRAY else .HEADER, 0);
}

toml_decode_document :: (r: *Toml_Reader, out: *$T) -> bool {
    root := toml_new_frame();

    defer toml_free_frame(root);

    while toml_gap(r) && r.pos < r.text.count {
        if toml_peek(r) == 91 {
            if !toml_decode_header(r, out, root) return false;
        } else if !toml_decode_pair(r, out, root, r.header_keys) return false;

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

    if r.error.kind != .NONE return false;

    r.path.count = 0;

    return toml_finish(r, out, root, 0);
}

// An inline table `{ ... }` read into a struct. Its members have their own declaration scope.
toml_inline :: (r: *Toml_Reader, out: *$T, message: string) -> bool {
    start := r.pos;

    if toml_peek(r) != 123 return toml_fail(r, .TYPE, ifx message.count then message else "expected a table", start);
    if !toml_enter_nested(r, "table") return false;

    frame := toml_new_frame();

    defer toml_free_frame(frame);
    r.pos += 1;

    if !toml_gap(r) return false;

    if toml_peek(r) != 125 {
        while true {
            if !toml_decode_pair(r, out, frame, .[]) || !toml_gap(r) return false;
            if toml_peek(r) == 125 break;

            if toml_peek(r) != 44 return toml_fail(r, .SYNTAX, "expected ',' or '}' in inline table");

            r.pos += 1;
            if !toml_gap(r) return false;
            if toml_peek(r) == 125 break;
        }
    }

    r.pos += 1;
    r.depth -= 1;

    return toml_finish(r, out, frame, start);
}

toml_missing_key :: (r: *Toml_Reader, key: string, at: int) -> bool {
    array_add(*r.path, Toml_Path_Part.{ name = key });

    return toml_fail(r, .TYPE, "missing required key", at);
}

// ---- Toml_Value fields ----
//
// A `Toml_Value` in a typed struct (or a `Hash_Table` value) is an untyped tree the target owns:
// every node, string and key is its own allocation, so nothing borrows the input and
// `free_toml_value` releases it. Assigned values (`x = ...`) are read by the document parser
// into a scratch document and copied out; tables made by headers and dotted keys are built here
// under the same declaration rules as every other typed table.

toml_new_node :: (kind: Toml_Kind, span: Toml_Span) -> *Toml_Value {
    node := New(Toml_Value);

    node.kind = kind;
    node.span = span;

    return node;
}

toml_copy_tree :: (source: *Toml_Value) -> *Toml_Value {
    node := toml_new_node(source.kind, source.span);

    if source.kind == {
        case .STRING;
            node.text = copy_string(source.text);
        case .INTEGER;
            node.integer = source.integer;
        case .FLOAT;
            node.real = source.real;
        case .BOOL;
            node.boolean = source.boolean;
        case .TABLE;
            array_reserve(*node.members, source.members.count);

            for source.members array_add(*node.members, Toml_Member.{ key = copy_string(it.key), value = toml_copy_tree(it.value), key_span = it.key_span });
        case .ARRAY;
            array_reserve(*node.items, source.items.count);

            for source.items array_add(*node.items, toml_copy_tree(it));
        case;
            node.temporal = source.temporal;
            node.temporal.fraction = copy_string(source.temporal.fraction);
    }

    return node;
}

// Releases what `node` owns and leaves an empty table; the node itself is the caller's.
toml_free_tree :: (node: *Toml_Value) {
    if node.kind == {
        case .STRING;
            if node.text.data free(node.text.data);
        case .TABLE;
            for node.members {
                if it.key.data free(it.key.data);

                if it.value {
                    toml_free_tree(it.value);
                    free(it.value);
                }
            }

            array_free(node.members);
        case .ARRAY;
            for node.items if it {
                toml_free_tree(it);
                free(it);
            }

            array_free(node.items);
        case .INTEGER;
        case .FLOAT;
        case .BOOL;
        case;
            if node.temporal.fraction.data free(node.temporal.fraction.data);
    }

    node.* = .{};
}

// Reads the value at the reader's position, of any shape, into a new tree. The document parser
// does the reading in a scratch reader that shares the text, the limits and the error path; its
// nodes are copied out and released.
toml_read_tree :: (r: *Toml_Reader) -> *Toml_Value {
    scratch: Toml_Reader;

    scratch.text = r.text;
    scratch.pos = r.pos;
    scratch.options = r.options;
    scratch.options.max_nodes = max(0, r.options.max_nodes - r.stream_nodes);
    scratch.options.record_statements = false;
    scratch.doc.options = scratch.options;
    scratch.doc.allocator = context.allocator;
    scratch.doc.source = r.doc.source;
    scratch.doc.retained_bytes = r.doc.retained_bytes;
    scratch.depth = r.depth;

    for r.path array_add(*scratch.path, Toml_Path_Part.{ name = it.name, index = it.index });

    value := toml_value(*scratch);
    copy: *Toml_Value;

    if value && scratch.error.kind == .NONE copy = toml_copy_tree(value);

    r.pos = scratch.pos;
    r.error = scratch.error;
    r.stream_nodes += scratch.doc.nodes.count;
    toml_free_scratch(*scratch);
    r.doc.retained_bytes = scratch.doc.retained_bytes;
    toml_free_nodes(*scratch.doc);

    return copy;
}

toml_read_value :: (r: *Toml_Reader, out: *Toml_Value) -> bool {
    copy := toml_read_tree(r);

    if !copy return false;

    out.* = copy.*;
    free(copy);

    return true;
}

// `[a.b]`, `[[a.b]]` and dotted keys below a `Toml_Value` table. `frame` holds a record per key,
// whose `member` is where the key's value sits in `node.members`.
toml_descend_value :: (r: *Toml_Reader, node: *Toml_Value, frame: *Toml_Frame, keys: [] string, index: int, op: Toml_Op, nav: int) -> bool {
    assert(node.kind == .TABLE);

    key := keys[index];
    step := toml_step(op, index, keys.count, nav);
    path_mark := r.path.count;

    array_add(*r.path, Toml_Path_Part.{ name = key });
    defer r.path.count = path_mark;

    entry := toml_frame_entry(r, frame, key);
    fresh := !entry.decl.declared;
    decl := *entry.decl;

    if !toml_enter(r, decl, step, r.statement_at) return false;

    child: *Toml_Value;

    if fresh {
        if step == .FINAL_ASSIGN {
            child = toml_read_tree(r);

            if !child return false;
        } else {
            child = toml_new_node(ifx step == .FINAL_ARRAY then Toml_Kind.ARRAY else .TABLE, .{ r.statement_at, r.pos });
        }

        entry.member = node.members.count;
        array_add(*node.members, Toml_Member.{ key = copy_string(key), value = child, key_span = .{ r.statement_at, r.pos } });
    } else {
        child = node.members[entry.member].value;
    }

    if step == .FINAL_ASSIGN return true;

    if step == .FINAL_ARRAY {
        if decl.child toml_free_frame(decl.child);

        decl.child = toml_new_frame();
        array_add(*child.items, toml_new_node(.TABLE, .{ r.statement_at, r.pos }));

        return true;
    }

    if !decl.child decl.child = toml_new_frame();

    if step == .FINAL_TABLE return true;

    table := child;

    // Below a `[[x]]` header the keys belong to its latest element.
    if decl.kind == .ARRAY_TABLE {
        array_add(*r.path, Toml_Path_Part.{ index = child.items.count - 1 });
        table = child.items[child.items.count - 1];
    }

    return toml_descend_value(r, table, decl.child, keys, index + 1, op, nav);
}

// ---- entry points ----

toml_decode_text :: (text: string, out: *$T, options: Toml_Options) -> Toml_Error {
    reader: Toml_Reader;
    reader.text = text;
    reader.options = options;
    reader.doc.options = options;
    reader.doc.allocator = context.allocator;
    reader.doc.source = text;
    reader.stream = true;
    reader.stream_nodes = 1;
    defer toml_free_stream(*reader);

    if !toml_options_valid(options) {
        toml_fail(*reader, .LIMIT, "limits must be nonnegative", 0);
        return reader.error;
    }

    if text.count > options.max_input_bytes {
        toml_fail(*reader, .LIMIT, "input byte limit exceeded", 0);
        return reader.error;
    }

    if !toml_validate_utf8(*reader) return reader.error;

    if !toml_decode_document(*reader, out) {
        defaults: T;

        toml_free_fields(out, *defaults);
        out.* = defaults;

        return reader.error;
    }

    return .{};
}

toml_free_stream :: (r: *Toml_Reader) {
    toml_clear_header_keys(r);
    array_free(r.path);
    array_free(r.header_keys);

    for *r.key_buffers array_reset(*it.values);

    array_reset(*r.key_buffers);
}
