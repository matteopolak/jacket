#scope_export
// Creates an empty owning document. Builder nodes belong to it until free_toml.
// Example: doc, error := new_toml_document();
new_toml_document :: (options := Toml_Options.{}) -> Toml_Document, Toml_Error {
    if !toml_options_valid(options) return .{}, .{ kind = .LIMIT, message = "limits must be nonnegative" };

    doc: Toml_Document;
    doc.options = options;
    doc.allocator = context.allocator;
    root, error := toml_new_value(*doc, .TABLE);
    if error.kind != .NONE {
        free_toml(*doc);
        return .{}, error;
    }

    doc.root = root;

    return doc, .{};
}

// Allocates a detached node. Its kind stays fixed; assign only scalar fields matching that kind.
// Use toml_set_text/toml_set_fraction for owned text, and toml_insert/toml_push to attach it.
// Example: value, error := toml_new_value(*doc, .INTEGER); value.integer = 7;
toml_new_value :: (doc: *Toml_Document, kind: Toml_Kind) -> *Toml_Value, Toml_Error {
    if !doc.allocator.proc return null, .{ kind = .OUTPUT, message = "document is not initialized" };

    push_allocator(doc.allocator);
    if doc.nodes.count >= doc.options.max_nodes return null, .{ kind = .LIMIT, message = "node limit exceeded" };

    growth, _ := toml_array_growth(*doc.nodes);
    if !toml_admit(doc, growth, toml_node_storage_growth(doc)) return null, .{ kind = .LIMIT, message = "retained byte limit exceeded" };

    toml_build_reserve(doc, *doc.nodes);
    value := toml_allocate_node(doc);
    value.kind = kind;
    if kind == .LOCAL_DATE || kind == .LOCAL_TIME || kind == .LOCAL_DATETIME || kind == .OFFSET_DATETIME value.temporal = .{};

    value.internal_index = doc.nodes.count;
    array_add(*doc.nodes, value);

    return value, .{};
}

// Copies a string with the document's allocator. Failure preserves both value and budget.
// max_string_bytes limits one decoded string/key/fraction; retained bytes bound their total.
// Example: error = toml_set_text(*doc, value, "tool");
toml_set_text :: (doc: *Toml_Document, value: *Toml_Value, text: string) -> Toml_Error {
    if !toml_belongs(doc, value) || value.kind != .STRING return .{ kind = .OUTPUT, message = "expected a string node in this document" };
    if text.count > doc.options.max_string_bytes return .{ kind = .LIMIT, message = "string byte limit exceeded" };

    invalid_at, invalid_message := toml_utf8_error(text);
    if invalid_at >= 0 {
        check: Toml_Reader;
        check.text = text;
        toml_fail(*check, .ENCODING, invalid_message, invalid_at);
        return check.error;
    }

    delta := text.count - value.text.count;
    if delta > 0 && !toml_admit(doc, delta) return .{ kind = .LIMIT, message = "retained byte limit exceeded" };

    push_allocator(doc.allocator);
    copy := copy_string(text);

    if value.text.data free(value.text);

    value.text = copy;
    doc.retained_bytes += delta;

    return .{};
}

// Copies exact fractional-second digits. Empty means no fractional component. Replacement
// temporarily holds both old and new strings; max_retained_bytes bounds the resulting document.
// Example: error = toml_set_fraction(*doc, value, "123456789012");
toml_set_fraction :: (doc: *Toml_Document, value: *Toml_Value, digits: string) -> Toml_Error {
    if !toml_belongs(doc, value) || (value.kind != .LOCAL_TIME && value.kind != .LOCAL_DATETIME && value.kind != .OFFSET_DATETIME) return .{ kind = .OUTPUT, message = "expected a temporal node in this document" };
    if digits.count > doc.options.max_string_bytes return .{ kind = .LIMIT, message = "fraction byte limit exceeded" };
    for digits if !toml_digit(it) return .{ kind = .OUTPUT, message = "fraction must contain decimal digits" };

    delta := digits.count - value.temporal.fraction.count;
    if delta > 0 && !toml_admit(doc, delta) return .{ kind = .LIMIT, message = "retained byte limit exceeded" };

    push_allocator(doc.allocator);
    copy := copy_string(digits);

    if value.temporal.fraction.data free(value.temporal.fraction);

    value.temporal.fraction = copy;
    doc.retained_bytes += delta;

    return .{};
}

// Inserts an absent key and a detached same-document node. Duplicates never replace silently.
// All depth/storage checks precede mutation. Node pointers remain stable across array/index growth.
// Example: error = toml_insert(*doc, doc.root, "count", value);
toml_insert :: (doc: *Toml_Document, table: *Toml_Value, key: string, value: *Toml_Value) -> Toml_Error {
    if !toml_can_attach(doc, table, value) || table.kind != .TABLE return .{ kind = .OUTPUT, message = "invalid table attachment" };
    if !toml_attachment_fits(doc, table, value) return .{ kind = .LIMIT, message = "document depth limit exceeded" };

    hint: Toml_Insert_Hint;
    if toml_probe_index(table, table.internal_lookup, key, *hint) >= 0 return .{ kind = .DUPLICATE, message = "key is already present" };
    if key.count > doc.options.max_string_bytes return .{ kind = .LIMIT, message = "key byte limit exceeded" };

    invalid_at, invalid_message := toml_utf8_error(key);
    if invalid_at >= 0 {
        check: Toml_Reader;
        check.text = key;
        toml_fail(*check, .ENCODING, invalid_message, invalid_at);
        return check.error;
    }

    if toml_prepare_insertion(table, table.internal_lookup, key, *hint) >= 0 return .{ kind = .DUPLICATE, message = "key is already present" };

    growth, _ := toml_array_growth(*table.members);
    if !toml_admit(doc, key.count, growth, toml_index_growth(table)) return .{ kind = .LIMIT, message = "retained byte limit exceeded" };

    push_allocator(doc.allocator);
    toml_build_reserve(doc, *table.members);
    toml_prepare_index(doc, table);
    owned := copy_string(key);

    doc.retained_bytes += key.count;
    array_add(*table.members, Toml_Member.{ key = owned, value = value });

    if table.internal_lookup && toml_tree_add(table, table.internal_lookup, table.members.count - 1, *hint, true) >= 0 {
        table.members.count -= 1;
        free(owned);
        doc.retained_bytes -= key.count;
        return .{ kind = .DUPLICATE, message = "key is already present" };
    }

    toml_note_child(table, value, table.members.count - 1);

    return .{};
}

// Appends a detached same-document node. Shared children, cycles and excessive depth are errors.
// Example: error = toml_push(*doc, list, value);
toml_push :: (doc: *Toml_Document, array: *Toml_Value, value: *Toml_Value) -> Toml_Error {
    if !toml_can_attach(doc, array, value) || array.kind != .ARRAY return .{ kind = .OUTPUT, message = "invalid array attachment" };
    if !toml_attachment_fits(doc, array, value) return .{ kind = .LIMIT, message = "document depth limit exceeded" };

    push_allocator(doc.allocator);
    if !toml_build_reserve(doc, *array.items) return .{ kind = .LIMIT, message = "retained byte limit exceeded" };

    array_add(*array.items, value);
    toml_note_child(array, value, array.items.count - 1);

    return .{};
}

#scope_module
Toml_Index_Node :: struct {
    left: int = -1;
    right: int = -1;
    height: int = 1;
}

Toml_Index :: struct {
    nodes: [..] Toml_Index_Node;
    root: int = -1;
}

toml_options_valid :: (options: Toml_Options) -> bool {
    return options.max_input_bytes >= 0 && options.max_depth >= 0 && options.max_nodes >= 0 && options.max_string_bytes >= 0 && options.max_number_bytes >= 0 && options.max_retained_bytes >= 0 && options.max_output_bytes >= 0;
}

// Pure admission: rejected operations cannot consume nonexistent bytes. Capacity changes commit
// only after allocation succeeds; allocator failure itself follows Basic's aborting contract.
toml_admit :: (doc: *Toml_Document, a: int, b: int = 0, c: int = 0, d: int = 0) -> bool {
    remaining := doc.options.max_retained_bytes - doc.retained_bytes;

    for int.[a, b, c, d] {
        if it < 0 || it > remaining return false;
        remaining -= it;
    }

    return true;
}

toml_array_growth :: (array: *[..] $T) -> int, int {
    if array.count < array.allocated return 0, array.allocated;
    if array.allocated > 0x3fffffffffffffff return -1, 0;

    capacity := max(8, array.allocated * 2);
    if capacity > 0x7fffffffffffffff / size_of(T) return -1, 0;

    return (capacity - array.allocated) * size_of(T), capacity;
}

toml_build_reserve :: (doc: *Toml_Document, array: *[..] $T) -> bool {
    growth, capacity := toml_array_growth(array);
    if !toml_admit(doc, growth) return false;

    if growth {
        array_reserve(array, capacity);
        doc.retained_bytes += growth;
    }

    return true;
}

toml_node_storage_growth :: (doc: *Toml_Document) -> int {
    return size_of(Toml_Value);
}

toml_allocate_node :: (doc: *Toml_Document) -> *Toml_Value {
    value := New(Toml_Value);
    doc.retained_bytes += size_of(Toml_Value);
    return value;
}

toml_belongs :: (doc: *Toml_Document, node: *Toml_Value) -> bool {
    return node && node.internal_index >= 0 && node.internal_index < doc.nodes.count && doc.nodes[node.internal_index] == node;
}

toml_can_attach :: (doc: *Toml_Document, parent: *Toml_Value, node: *Toml_Value) -> bool {
    if !toml_belongs(doc, parent) || !toml_belongs(doc, node) || node.internal_parent || node == doc.root return false;

    ancestor := parent;
    steps := 0;

    while ancestor {
        if ancestor == node || steps >= doc.nodes.count return false;
        ancestor = ancestor.internal_parent;
        steps += 1;
    }

    return true;
}

toml_attachment_fits :: (doc: *Toml_Document, parent: *Toml_Value, node: *Toml_Value) -> bool {
    depth := 0;
    ancestor := parent.internal_parent;

    while ancestor {
        depth += 1;
        ancestor = ancestor.internal_parent;
    }

    return depth < doc.options.max_depth && node.internal_height < doc.options.max_depth - depth;
}

toml_note_child :: (parent: *Toml_Value, node: *Toml_Value, slot: int) {
    node.internal_parent = parent;
    node.internal_parent_slot = slot;
    height := node.internal_height + 1;
    ancestor := parent;

    while ancestor && height > ancestor.internal_height {
        ancestor.internal_height = height;
        height += 1;
        ancestor = ancestor.internal_parent;
    }
}

toml_member :: (table: *Toml_Value, key: string) -> *Toml_Member {
    if !table || table.kind != .TABLE return null;

    if table.internal_lookup {
        index := toml_tree_find(table, table.internal_lookup, key);
        if index >= 0 return *table.members[index];
        return null;
    }

    for *table.members if it.key == key return it;

    return null;
}

// A deterministic balanced index avoids hash-collision attacks. Its node number is the
// insertion-order member number, so growing either array never invalidates value pointers.
toml_index_capacity :: (table: *Toml_Value) -> int {
    assert(table.kind == .TABLE);

    if table.members.count == 0x7fffffffffffffff return -1;

    desired := table.members.count + 1;
    if desired < 16 return 0;

    capacity := ifx table.internal_lookup then table.internal_lookup.nodes.allocated else 16;
    while capacity < desired {
        if capacity > 0x3fffffffffffffff return -1;
        capacity *= 2;
    }

    return capacity;
}

toml_index_growth :: (table: *Toml_Value) -> int {
    capacity := toml_index_capacity(table);
    if capacity < 0 || capacity > 0x7fffffffffffffff / size_of(Toml_Index_Node) return -1;
    if !capacity return 0;

    previous := ifx table.internal_lookup then table.internal_lookup.nodes.allocated else 0;
    bytes := (capacity - previous) * size_of(Toml_Index_Node);

    if !table.internal_lookup {
        if bytes > 0x7fffffffffffffff - size_of(Toml_Index) return -1;
        bytes += size_of(Toml_Index);
    }

    return bytes;
}

toml_prepare_index :: (doc: *Toml_Document, table: *Toml_Value) -> bool {
    assert(table.kind == .TABLE);

    growth := toml_index_growth(table);
    if !toml_admit(doc, growth) return false;
    if !growth return true;

    capacity := toml_index_capacity(table);
    index := table.internal_lookup;

    if !index {
        index = New(Toml_Index);
        array_reserve(*index.nodes, capacity);
        for table.members toml_tree_add(table, index, it_index);
        table.internal_lookup = index;
    } else array_reserve(*index.nodes, capacity);

    doc.retained_bytes += growth;

    return true;
}
