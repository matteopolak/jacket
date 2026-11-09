#import "Basic";
#import "Extensions/Toml";

// The typed encoder and the DOM writer refuse the same documents at every depth limit: the root
// is depth zero, each value is one deeper than its table or array, and an array of tables adds
// a level for the array and one for its elements.
Leaf :: struct {
    v: int;
}

Mid :: struct {
    n: int;
    leaf: Leaf;
    rows: [..] Leaf;
    list: [..] int;
    deep: [..][..] int;
}

Root :: struct {
    a: int;
    mid: Mid;
}

Flat :: struct {
    a: int;
}

Empty :: struct {
    leaf: Leaf;
}

compare :: (value: *$T) {
    text, error := toml_encode(value);

    assert(error.kind == .NONE, error.message);
    defer free(text);

    doc, parse_error := parse_toml(text);

    assert(parse_error.kind == .NONE, text);
    defer free_toml(*doc);

    for depth: 0..7 {
        dom, dom_error := write_toml(doc.root, .{ max_depth = depth });
        typed, typed_error := toml_encode(value, .{ max_depth = depth });

        assert(dom_error.kind == typed_error.kind, tprint("depth % of\n%DOM %, typed %", depth, text, dom_error.kind, typed_error.kind));
        if dom_error.kind == .NONE assert(dom == typed, tprint("depth %: % != %", depth, dom, typed));

        free(dom);
        free(typed);
    }
}

Hollow :: struct {}

Holder :: struct {
    hollow: Hollow;
}

Nothing :: struct {
    a: int; @toml(skip_serializing)
}

Stamped :: struct {
    when: Toml_Datetime;
}

main :: () {
    // Tables with nothing in them sit at the limit without going past it.
    nothing: Nothing;
    holder: Holder;
    stamped := Stamped.{ .{ .LOCAL_DATE, .{ year = 2020, month = 1, day = 2 } } };

    compare(*nothing);
    compare(*holder);
    compare(*stamped);

    flat := Flat.{ 1 };

    compare(*flat);

    empty: Empty;

    compare(*empty);

    root: Root;

    root.a = 1;
    root.mid.n = 2;
    array_add(*root.mid.rows, Leaf.{ 1 });
    array_add(*root.mid.rows, Leaf.{ 2 });
    array_add(*root.mid.list, 3);

    inner: [..] int;

    array_add(*inner, 4);
    array_add(*root.mid.deep, inner);

    compare(*root);
    // `root.mid.deep` holds `inner`, so this frees it too.
    free_toml_value(*root);
    print("ok\n");
}
