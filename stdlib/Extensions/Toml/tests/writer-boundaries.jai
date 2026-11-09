#import "Basic";
#import "Extensions/Toml";

// Output and nesting limits fail at the same place, with the same error, in the document
// writer and the typed encoder, for a document made of brackets, separators, headers and
// nested arrays. The sweep tries every byte limit from nothing to the full size.
Leaf :: struct {
    n: int;
}

Inner :: struct {
    list: [3] int;
    leaf: Leaf;
}

Doc :: struct {
    title: string;
    nested: [2][2] int;
    inner: Inner;
    rows: [..] Leaf;
}

// The deepest value is an empty array, which only the nesting limit can refuse.
Grid :: struct {
    grid: [..][..] int;
}

same_error :: (a: *Toml_Error, b: *Toml_Error, what: string) {
    assert(a.kind == b.kind, what);
    assert(a.message == b.message, what);
    assert(toml_error_path(a) == toml_error_path(b), tprint("% dom % typed %", what, toml_error_path(a), toml_error_path(b)));
}

check_bytes :: (doc: *Toml_Document, value: *$T, size: int) {
    for limit: 0..size + 1 {
        options := Toml_Options.{ max_output_bytes = limit };
        dom, dom_error := write_toml(doc.root, options);
        typed, typed_error := toml_encode(value, options);
        what := tprint("max_output_bytes = %", limit);

        if limit >= size {
            assert(dom_error.kind == .NONE && typed_error.kind == .NONE, what);
            assert(dom == typed && dom.count == size, what);
            free(dom);
            free(typed);
        } else {
            assert(dom_error.kind == .LIMIT && typed_error.kind == .LIMIT, what);
            same_error(*dom_error, *typed_error, what);
        }
    }
}

check_depth :: (doc: *Toml_Document, value: *$T, depth: int) {
    for limit: depth - 1..depth {
        options := Toml_Options.{ max_depth = limit };
        dom, dom_error := write_toml(doc.root, options);
        typed, typed_error := toml_encode(value, options);
        what := tprint("max_depth = %", limit);

        if limit >= depth {
            assert(dom_error.kind == .NONE && typed_error.kind == .NONE, what);
            assert(dom == typed, what);
            free(dom);
            free(typed);
        } else {
            assert(dom_error.kind == .LIMIT && typed_error.kind == .LIMIT, what);
            assert(dom_error.message == "writer nesting limit exceeded", what);
            same_error(*dom_error, *typed_error, what);
        }
    }
}

main :: () {
    value := Doc.{ title = "t\"q" };

    value.nested[0] = .[1, 2];
    value.nested[1] = .[3, 4];
    value.inner.list = .[5, 6, 7];
    value.inner.leaf.n = 8;
    array_add(*value.rows, Leaf.{ 9 });
    array_add(*value.rows, Leaf.{ 10 });
    defer array_free(value.rows);

    text, error := toml_encode(*value);

    assert(error.kind == .NONE, error.message);
    defer free(text);

    doc, parse_error := parse_toml(text);

    assert(parse_error.kind == .NONE, parse_error.message);
    defer free_toml(*doc);

    check_bytes(*doc, *value, text.count);
    check_depth(*doc, *value, 3);

    grid: Grid;
    empty: [..] int;

    array_add(*grid.grid, empty);
    defer array_free(grid.grid);

    grid_text, grid_error := toml_encode(*grid);

    assert(grid_error.kind == .NONE, grid_error.message);
    defer free(grid_text);

    grid_doc, grid_parse := parse_toml(grid_text);

    assert(grid_parse.kind == .NONE, grid_parse.message);
    defer free_toml(*grid_doc);

    check_bytes(*grid_doc, *grid, grid_text.count);
    check_depth(*grid_doc, *grid, 2);
    print("ok\n");
}
