// General TOML 1.1 values, parsing and semantic writing. The core only consumes memory.
// Documents own their source, nodes and decoded strings; free_toml releases them together.
//
// ```jai
// doc, error := parse_toml("name = 'tool'\n");
// assert(error.kind == .NONE);
// assert(toml_get(doc.root, "name").text == "tool");
// free_toml(*doc);
// ```

// The value stored in a TOML node; temporal spellings retain their distinct kinds.
Toml_Kind :: enum u8 {
    TABLE;
    ARRAY;
    STRING;
    INTEGER;
    FLOAT;
    BOOL;
    LOCAL_DATE;
    LOCAL_TIME;
    LOCAL_DATETIME;
    OFFSET_DATETIME;
}

// Half-open byte offsets in the document source; -1 means no source location.
Toml_Span :: struct {
    start: int = -1;
    end: int = -1;
}

// Failure category; NONE means parsing or writing succeeded.
Toml_Error_Kind :: enum u8 {
    NONE;
    SYNTAX;
    TYPE;
    DUPLICATE;
    ENCODING;
    RANGE;
    LIMIT;
    OUTPUT;
}

// Messages are static; errors remain valid after input/document release. Columns count Unicode
// scalars from one, tabs count one, and spans are half-open byte ranges. related is spanless
// unless an earlier declaration caused the error.
Toml_Error :: struct {
    kind: Toml_Error_Kind;
    message: string;
    span: Toml_Span;
    related: Toml_Span;
    line: int;
    column: int;
    path_bytes: [512] u8;
    path_count: int;
    path_truncated: bool;
}

// The error's allocation-independent path, borrowed from the error record. Truncation is
// explicit in path_truncated; full byte spans remain authoritative.
// Example: assert(toml_error_path(*error) == "$[\"format\"][\"indent_width\"]");
toml_error_path :: (error: *Toml_Error) -> string {
    return .{ error.path_count, error.path_bytes.data };
}

// Civil fields never consult a timezone. Fractions retain all digits. :60 is retained as a
// leap-second spelling; this format layer does not verify historical leap insertion events.
Toml_Temporal :: struct {
    year: int;
    month: int;
    day: int;
    hour: int;
    minute: int;
    second: int;
    fraction: string;
    offset_minutes: int;
    negative_zero_offset: bool;
}

// One decoded table key and its document-owned value, in insertion order.
Toml_Member :: struct {
    key: string;
    value: *Toml_Value;
    key_span: Toml_Span;
}

// Nodes are document-owned and stay at the same address. Do not copy ownership, attach nodes
// from another document or introduce cycles. Members/items are insertion-ordered; their array
// storage may move on growth. Parsed values' spans refer to document.source.
Toml_Value :: struct {
    kind: Toml_Kind;
    union {
        members: [..] Toml_Member;
        text: string;
        integer: s64;
        real: float64;
        boolean: bool;
        temporal: Toml_Temporal;
        items: [..] *Toml_Value;
    }
    span: Toml_Span;
    internal_index: int;
    internal_parent: *Toml_Value;
    internal_lookup: *Toml_Index;
    internal_parent_slot: int;
    internal_height: int;
}

// Memory, nesting and output limits shared by parsing and writing.
// Set record_statements before parsing to retain source statement order.
Toml_Options :: struct {
    max_input_bytes: int = 16 << 20;
    max_depth: int = 128;
    max_nodes: int = 1 << 20;
    max_string_bytes: int = 16 << 20; // one decoded key, string or fraction
    record_statements: bool; // optional source projection, used by Jai_Format
    max_number_bytes: int = 4096;
    max_retained_bytes: int = 128 << 20;
    max_output_bytes: int = 32 << 20;
}

// Owns the source, nodes and decoded strings; release with free_toml.
// Node pointers and statement records remain valid until that release.
Toml_Document :: struct {
    root: *Toml_Value;
    source: string;
    nodes: [..] *Toml_Value;
    allocator: Allocator;
    options: Toml_Options;
    retained_bytes: int;
    statements: [..] Toml_Statement;
}

// Looks up one exact decoded key; null means absent or a non-table. Dots are literal here.
// Example: toml_get(doc.root, "format") finds the format table.
toml_get :: (table: *Toml_Value, key: string) -> *Toml_Value {
    if !table || table.kind != .TABLE return null;
    member := toml_member(table, key);
    if member return member.value;
    return null;
}

// Follows exact key segments. Absence is null/NONE; traversing a scalar is TYPE. Nodes remain
// document-owned. Lookup errors have byte spans where available and no computed line/column.
// Example: value, error := toml_get_path(doc.root, string.["format", "indent_width"]);
toml_get_path :: (table: *Toml_Value, keys: [] string) -> *Toml_Value, Toml_Error {
    value := table;

    for keys {
        if !value || value.kind != .TABLE {
            error: Toml_Error;
            error.kind = .TYPE;
            error.message = "cannot look up a key through a non-table";
            if value error.span = value.span;

            parts: [128] Toml_Path_Part;
            count := min(it_index + 1, parts.count);

            for index: 0..count - 1 parts[index] = .{ name = keys[index] };

            reader: Toml_Reader;
            reader.path.data = parts.data;
            reader.path.count = count;
            toml_render_path(*reader, *error);
            if count < it_index + 1 error.path_truncated = true;

            return null, error;
        }

        value = toml_get(value, it);
        if !value return null, .{};
    }

    return value, .{};
}

// Releases the complete document with its original allocator; safe to call again.
// Example: free_toml(*doc);
free_toml :: (doc: *Toml_Document) {
    if !doc.nodes.data && !doc.source.data {
        doc.* = .{};
        return;
    }

    push_allocator(doc.allocator);

    for node: doc.nodes {
        if node.kind == .STRING {
            if node.text.data free(node.text);
        } else if node.kind == .TABLE {
            if node.internal_lookup {
                array_free(node.internal_lookup.nodes);
                free(node.internal_lookup);
            }

            for member: node.members if member.key.data && !toml_key_uses_source(doc.source, member.key) free(member.key);

            array_free(node.members);
        } else if node.kind == .ARRAY array_free(node.items);
        else if node.kind == .LOCAL_DATE || node.kind == .LOCAL_TIME || node.kind == .LOCAL_DATETIME || node.kind == .OFFSET_DATETIME {
            if node.temporal.fraction.data free(node.temporal.fraction);
        }

        free(node);
    }

    array_free(doc.nodes);
    array_free(doc.statements);
    if doc.source.data free(doc.source);

    doc.* = .{};
}

// Parses a complete TOML 1.1 document atomically. All returned storage is independent of input.
// Example: doc, error := parse_toml("enabled = true\n");
parse_toml :: (text: string, options := Toml_Options.{}) -> Toml_Document, Toml_Error {
    reader: Toml_Reader;
    reader.text = text;
    reader.options = options;
    reader.doc.options = options;
    reader.doc.allocator = context.allocator;
    defer toml_free_scratch(*reader);

    if !toml_options_valid(options) {
        toml_fail(*reader, .LIMIT, "limits must be nonnegative", 0);
        return .{}, reader.error;
    }

    if text.count > options.max_input_bytes {
        toml_fail(*reader, .LIMIT, "input byte limit exceeded", 0);
        return .{}, reader.error;
    }

    if !toml_validate_utf8(*reader) return .{}, reader.error;

    if !toml_admit(*reader.doc, text.count) {
        toml_fail(*reader, .LIMIT, "retained byte limit exceeded", 0);
        return .{}, reader.error;
    }

    reader.doc.source = copy_string(text);
    reader.doc.retained_bytes += text.count;
    reader.text = reader.doc.source;
    reader.doc.root = toml_node(*reader, .TABLE);
    if reader.doc.root && toml_document(*reader) {
        toml_free_scratch(*reader);
        return reader.doc, .{};
    }

    toml_free_scratch(*reader);
    free_toml(*reader.doc);

    return .{}, reader.error;
}

#scope_module
#import "Basic";
#import "String";
#load "build.jai";
#load "key-storage.jai";
#load "index.jai";
#load "reader.jai";
#load "statements.jai";
#load "scalar.jai";
#load "writer.jai";
