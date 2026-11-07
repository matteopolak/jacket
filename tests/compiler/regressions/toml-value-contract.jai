// The browser engine and native/WASI regression catalog exercise the same portable consumer.
#import "Basic";
#import "Extensions/Toml";
F :: #import "Extensions/Jai_Format";

main :: () {
    doc, error := parse_toml("[format]\nindent_width = 0x2\n[other]\nnegative = -0.0\nminimum = 5e-324\nat = 2024-02-29T12:34:56.123456789012-00:00\n", .{ record_statements = true });

    assert(error.kind == .NONE, error.message);
    defer free_toml(*doc);
    assert(doc.statements.count == 6);
    selected, path_error := toml_get_path(doc.root, string.["other", "at"]);

    assert(path_error.kind == .NONE && selected.temporal.fraction == "123456789012");
    assert(selected.temporal.negative_zero_offset);
    output, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(output);
    again, again_error := parse_toml(output);

    assert(again_error.kind == .NONE);
    defer free_toml(*again);
    assert(toml_get(toml_get(again.root, "other"), "at").temporal.fraction == "123456789012");
    other := toml_get(again.root, "other");

    assert((cast,force(u64) toml_get(other, "negative").real) == 0x8000000000000000);
    assert((cast,force(u64) toml_get(other, "minimum").real) == 1);
    config, config_ok, config_error := F.parse_config(output);

    assert(config_ok && config.indent_width == 2, config_error);
    _, bad_config, _ := F.parse_config("[other]\nbroken = [\n");

    assert(!bad_config);
    _, duplicate := parse_toml("[a.b]\nx=1\n[a]\nb.y=2\n[a.b]\nz=3\n");

    assert(duplicate.kind == .DUPLICATE);
    builder: String_Builder;
    for 0..32 print_to_builder(*builder, "key% = %\n", formatInt(it), formatInt(it));

    append(*builder, "nested = {child = {");

    for 0..79 {
        if it append(*builder, ", ");
        print_to_builder(*builder, "field% = [%]", formatInt(it), formatInt(it));
    }

    append(*builder, "}}\n");
    wide_text := builder_to_string(*builder);

    free_buffers(*builder);
    wide, wide_error := parse_toml(wide_text);

    assert(wide_error.kind == .NONE, wide_error.message);
    for 0..32 assert(toml_get(wide.root, tprint("key%", formatInt(it))).integer == it);

    child := toml_get(toml_get(wide.root, "nested"), "child");

    assert(toml_get(child, "field79").items[0].integer == 79);
    free_toml(*wide);
    free(wide_text);
    print("ok\n");
}
