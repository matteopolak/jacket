#import "Basic";
#import "String";
#import "Extensions/Toml";

Write_Stats :: struct {
    bytes: int;
}

count_write :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Write_Stats) data;
    if mode == .ALLOCATE || mode == .RESIZE stats.bytes += size;
    backing := context.default_allocator;
    return backing.proc(mode, size, old_size, memory, backing.data);
}

counted_write :: (root: *Toml_Value) -> string, Toml_Error, int {
    stats: Write_Stats;
    text: string;
    error: Toml_Error;

    {
        push_allocator(count_write, *stats);
        text, error = write_toml(root);
    }

    return text, error, stats.bytes;
}

make_wide :: (count: int) -> Toml_Document {
    doc, error := new_toml_document();

    assert(error.kind == .NONE);

    for 0..count - 1 {
        value, value_error := toml_new_value(*doc, .INTEGER);

        assert(value_error.kind == .NONE);
        value.integer = it;
        key := tprint("key_%", it);

        assert(toml_insert(*doc, doc.root, key, value).kind == .NONE);
    }

    return doc;
}

main :: () {
    // Indexed and raw tables have identical output. An existing checked index must avoid
    // allocating a second full key tree on every write; the raw fallback still validates.
    doc := make_wide(256);
    raw := doc.root.*;
    raw.internal_lookup = null;
    indexed, indexed_error, indexed_bytes := counted_write(doc.root);
    plain, plain_error, plain_bytes := counted_write(*raw);

    assert(indexed_error.kind == .NONE && plain_error.kind == .NONE);
    assert(indexed == plain);
    free(indexed);
    free(plain);
    free_toml(*doc);

    for count: int.[16, 17, 64] {
        doc = make_wide(count);
        expected, expected_error := write_toml(doc.root);

        assert(expected_error.kind == .NONE);
        index := doc.root.internal_lookup;

        assert(index && index.nodes.count == count);
        saved_root := index.root;
        saved_node := index.nodes[saved_root];

        for variant: 0..5 {
            if variant == 0 index.nodes[saved_root].left = saved_root; // cycle
            else if variant == 1 index.nodes[saved_root].left = count; // out of range
            else if variant == 2 index.nodes[saved_root].left = -1; // missing subtree
            else if variant == 3 index.nodes[saved_root].right = saved_node.left; // repeated child
            else if variant == 4 index.root = count;
            else index.nodes.count = count - 1;

            text, error := write_toml(doc.root);

            assert(error.kind == .NONE && text == expected);
            free(text);
            index.root = saved_root;
            index.nodes.count = count;
            index.nodes[saved_root] = saved_node;
        }

        saved_key := doc.root.members[0].key;

        doc.root.members[0].key = "renamed";
        renamed, renamed_error := write_toml(doc.root);

        assert(renamed_error.kind == .NONE && contains(renamed, "\"renamed\" = 0"));
        free(renamed);
        doc.root.members[0].key = doc.root.members[1].key;
        _, duplicate := write_toml(doc.root);

        assert(duplicate.kind == .OUTPUT);
        doc.root.members[0].key = saved_key;
        _, limited := write_toml(doc.root, .{ max_nodes = 3 });

        assert(limited.kind == .LIMIT && toml_error_path(*limited) == "$[\"key_2\"]");
        _, output_limited := write_toml(doc.root, .{ max_output_bytes = 1 });

        assert(output_limited.kind == .LIMIT);
        free(expected);
        free_toml(*doc);
    }

    check_string_spans();
    assert(indexed_bytes < plain_bytes, tprint("indexed=% raw=%", indexed_bytes, plain_bytes));
    print("ok\n");
}

check_string_spans :: () {
    bytes: [260] u8;
    for *bytes it.* = 97;

    bytes[63] = 34;
    bytes[64] = 92;
    bytes[127] = 10;
    bytes[128] = 127;
    bytes[191] = 0xc2;
    bytes[192] = 0xa3;
    bytes[255] = 9;
    input := string.{ bytes.count, bytes.data };
    doc, _ := new_toml_document();
    value, _ := toml_new_value(*doc, .STRING);

    assert(toml_set_text(*doc, value, input).kind == .NONE);
    assert(toml_insert(*doc, doc.root, "spans", value).kind == .NONE);
    output, error := write_toml(doc.root);

    assert(error.kind == .NONE);
    parsed, parse_error := parse_toml(output);

    assert(parse_error.kind == .NONE && toml_get(parsed.root, "spans").text == input);
    free(output);
    free_toml(*parsed);
    free_toml(*doc);

    bad: Toml_Value;
    bad.kind = .STRING;
    bytes[0] = 255;
    bad.text = input;
    raw: Toml_Value;

    array_add(*raw.members, Toml_Member.{ key = "bad", value = *bad });
    _, encoding := write_toml(*raw);

    assert(encoding.kind == .OUTPUT && toml_error_path(*encoding) == "$[\"bad\"]");
    array_free(raw.members);
}
