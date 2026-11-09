#import "Basic";
#import "Extensions/Toml";

// The typed encoder and the DOM writer count the same nodes (the root, every value and every
// array), and a float32 is written with its own shortest digits.
Three :: struct {
    a: int;
    b: int;
    c: int;
}

Singles :: struct {
    tenth: float32;
    half: float32;
    biggest: float32;
    list: [] float32;
}

Rows :: struct {
    n: int;
}

Table :: struct {
    rows: [..] Rows;
}

check_limit :: (text: string, value: *$T, nodes: int) {
    doc, error := parse_toml(text);

    assert(error.kind == .NONE, error.message);
    defer free_toml(*doc);

    // One node fewer than the document has is refused by both writers, the exact count is enough.
    dom, dom_error := write_toml(doc.root, .{ max_nodes = nodes });

    assert(dom_error.kind == .NONE, dom_error.message);
    free(dom);
    _, dom_limit := write_toml(doc.root, .{ max_nodes = nodes - 1 });

    assert(dom_limit.kind == .LIMIT, text);

    typed, typed_error := toml_encode(value, .{ max_nodes = nodes });

    assert(typed_error.kind == .NONE, tprint("% nodes: %", nodes, typed_error.message));
    free(typed);
    _, typed_limit := toml_encode(value, .{ max_nodes = nodes - 1 });

    assert(typed_limit.kind == .LIMIT, tprint("% nodes should not fit %", nodes - 1, text));
}

main :: () {
    three := Three.{ 1, 2, 3 };

    check_limit("a = 1\nb = 2\nc = 3\n", *three, 4);

    table: Table;

    array_add(*table.rows, Rows.{ 1 });
    array_add(*table.rows, Rows.{ 2 });
    check_limit("[[rows]]\nn = 1\n[[rows]]\nn = 2\n", *table, 6);
    free_toml_value(*table);

    values: [3] float32 = .[0.5, 0.25, 8.0];
    max_bits: u32 = 0x7f7fffff;
    singles := Singles.{ 0.1, 0.5, cast,force(float32) max_bits, .{ 3, values.data } };
    text, error := to_toml(singles);

    assert(error.kind == .NONE, error.message);
    defer free(text);
    assert(text == "tenth = 0.1\nhalf = 0.5\nbiggest = 3.4028235e+38\nlist = [0.5, 0.25, 8.0]\n", text);
    print("ok\n");
}
