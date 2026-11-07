#import "Basic";
#import "Extensions/Toml";

main :: () {
    doc, error := parse_toml("format = { extra.deep = {x=1}, indent_width = 2 }\n[doc.format]\nmax_width = 80\n", .{ record_statements = true });

    assert(error.kind == .NONE, error.message);
    assert(doc.statements.count == 6);
    expected := string.["format", "format.extra.deep", "format.extra.deep.x", "format.indent_width", "doc.format", "doc.format.max_width"];

    for doc.statements {
        buffer: [128] string;
        path, path_error := toml_statement_path(*doc, it, buffer);

        assert(path_error.kind == .NONE, path_error.message);
        builder: String_Builder;

        for path {
            if it_index append(*builder, ".");
            append(*builder, it);
        }

        text := builder_to_string(*builder);

        assert(text == expected[it_index], text);
        free(text);
        free_buffers(*builder);
    }

    assert(doc.statements[0].span.start == 0 && doc.statements[0].span.end > doc.statements[2].span.end);
    assert(doc.statements[4].kind == .TABLE && doc.statements[4].line == 2 && doc.statements[5].line == 3);
    small: [1] string;
    _, too_small := toml_statement_path(*doc, doc.statements[2], small);

    assert(too_small.kind == .LIMIT);
    free_toml(*doc);

    arrays, array_error := parse_toml("[[items]]\nvalue={a=1}\n[[items]]\nvalue={b=2}\n", .{ record_statements = true });

    assert(array_error.kind == .NONE);
    assert(arrays.statements.count == 6);
    assert(arrays.statements[0].value != arrays.statements[3].value);
    buffer: [128] string;
    path, path_error := toml_statement_path(*arrays, arrays.statements[5], buffer);

    assert(path_error.kind == .NONE && path.count == 3 && path[0] == "items" && path[1] == "value" && path[2] == "b");
    free_toml(*arrays);
    print("ok\n");
}
