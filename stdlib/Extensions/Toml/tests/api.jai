#import "Basic";
#import "Extensions/Toml";

main :: () {
    doc, error := parse_toml("title = 'settings'\n[format]\nindent_width = 2\n");

    assert(error.kind == .NONE, error.message);
    format := toml_get(doc.root, "format");

    assert(format && toml_get(format, "indent_width").integer == 2);
    assert(toml_get(doc.root, "title").text == "settings");
    selected, path_error := toml_get_path(doc.root, string.["format", "indent_width"]);

    assert(path_error.kind == .NONE && selected.integer == 2);
    missing, missing_error := toml_get_path(doc.root, string.["format", "missing"]);

    assert(missing == null && missing_error.kind == .NONE);
    _, scalar_path := toml_get_path(doc.root, string.["title", "child"]);

    assert(scalar_path.kind == .TYPE);
    text, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE, write_error.message);
    again, again_error := parse_toml(text);

    assert(again_error.kind == .NONE, again_error.message);
    assert(toml_get(toml_get(again.root, "format"), "indent_width").integer == 2);
    free_toml(*again);
    free(text);
    free_toml(*doc);
    free_toml(*doc);

    parents, parents_error := parse_toml("[a.b]\nx = 1\n[a]\ny = 2\n");

    assert(parents_error.kind == .NONE, parents_error.message);
    free_toml(*parents);
    invalid := string.[
        "a = 1\n\"a\" = 2\n",
        "a.b = 1\n[a]\n",
        "a = { b = 1 }\na.c = 2\n",
        "[a]\n[a]\n",
        "a = []\n[[a]]\n",
        "a = 1\na.b = 2\n",
        "a = { b = 1, b = 2 }\n",
    ];
    for invalid {
        rejected, failure := parse_toml(it);
        assert(failure.kind == .DUPLICATE, it);
        assert(rejected.root == null);
    }

    duplicate, duplicate_error := parse_toml("a = 1\na = 2\n");

    assert(duplicate_error.span.start == 6 && duplicate_error.related.start == 0);
    assert(duplicate_error.line == 2 && duplicate_error.column == 1);
    assert(duplicate.root == null);

    temporal, temporal_error := parse_toml("at = 1979-05-27T07:32:00.123456789012-00:00\ntime = 07:32\n");

    assert(temporal_error.kind == .NONE, temporal_error.message);
    assert(toml_get(temporal.root, "at").temporal.fraction == "123456789012");
    assert(toml_get(temporal.root, "at").temporal.negative_zero_offset);
    assert(toml_get(temporal.root, "time").kind == .LOCAL_TIME);
    temporal_text, temporal_write := write_toml(temporal.root);

    assert(temporal_write.kind == .NONE);
    temporal_again, temporal_again_error := parse_toml(temporal_text);

    assert(temporal_again_error.kind == .NONE);
    assert(toml_get(temporal_again.root, "at").temporal.fraction == "123456789012");
    assert(toml_get(temporal_again.root, "at").temporal.negative_zero_offset);
    free_toml(*temporal_again);
    free(temporal_text);
    free_toml(*temporal);
    limited, limit_error := parse_toml("x = 1", .{ max_input_bytes = 4 });

    assert(limit_error.kind == .LIMIT && limited.root == null);
    print("ok\n");
}
