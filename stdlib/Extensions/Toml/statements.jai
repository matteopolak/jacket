#scope_export
// Source statements are optional: record_statements enables this ordered projection without a
// second parser. Inline assignments are included; an outer assignment precedes its children.
Toml_Statement_Kind :: enum u8 {
    ASSIGNMENT;
    TABLE;
    ARRAY_TABLE;
}

// One optional source projection record; spans and values belong to its document.
Toml_Statement :: struct {
    kind: Toml_Statement_Kind;
    value: *Toml_Value;
    span: Toml_Span;
    header_depth: int; // named segments in the enclosing document-level header
    line: int;
}

// Copies a statement's decoded key segments into caller storage, borrowing the strings from doc.
// Array element indices are omitted. The projection describes the parsed source;
// statements and returned keys expire with the document. Internal parent/slot fields are read-only.
// Example: path, error := toml_statement_path(*doc, doc.statements[0], buffer);
toml_statement_path :: (doc: *Toml_Document, statement: Toml_Statement, buffer: [] string) -> [] string, Toml_Error {
    if !toml_belongs(doc, statement.value) return .{}, .{ kind = .OUTPUT, message = "statement does not belong to this document" };

    current := statement.value;
    count := 0;
    steps := 0;

    while current && current.internal_parent {
        if !toml_belongs(doc, current) || steps >= doc.nodes.count return .{}, .{ kind = .OUTPUT, message = "invalid statement projection" };

        parent := current.internal_parent;
        if parent.kind == .TABLE {
            slot := current.internal_parent_slot;
            if slot < 0 || slot >= parent.members.count || parent.members[slot].value != current return .{}, .{ kind = .OUTPUT, message = "invalid statement parent slot" };
            if count >= buffer.count return .{}, .{ kind = .LIMIT, message = "statement path buffer is too small" };

            buffer[count] = parent.members[slot].key;
            count += 1;
        }

        current = parent;
        steps += 1;
    }

    if current != doc.root return .{}, .{ kind = .OUTPUT, message = "statement does not belong to this document" };

    for 0..count / 2 - 1 {
        key := buffer[it];
        buffer[it] = buffer[count - it - 1];
        buffer[count - it - 1] = key;
    }

    return .{ count, buffer.data }, .{};
}

#scope_module
toml_begin_statement :: (r: *Toml_Reader, kind: Toml_Statement_Kind, start: int, header_depth: int) -> int {
    if !r.options.record_statements return -1;
    if !toml_reserve(r, *r.doc.statements) return -2;

    while r.line_scan < start {
        if r.text[r.line_scan] == 10 r.statement_line += 1;
        r.line_scan += 1;
    }

    index := r.doc.statements.count;

    array_add(*r.doc.statements, Toml_Statement.{ kind = kind, span = .{ start, -1 }, header_depth = header_depth, line = r.statement_line });

    return index;
}
