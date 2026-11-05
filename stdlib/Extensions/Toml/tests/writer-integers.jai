#import "Basic";
#import "Extensions/Toml";

check_integer :: (doc: *Toml_Document, value: s64) {
    node := toml_get(doc.root, "x");
    node.integer = value;
    expected := tprint("x = %\n", formatInt(value));
    before := get_temporary_storage_mark();
    text, error := write_toml(doc.root);
    after := get_temporary_storage_mark();

    assert(error.kind == .NONE && text == expected, text);
    // Format-defined decimal needs no temporary result string or general print builder.
    assert(before.total_bytes_occupied == after.total_bytes_occupied, "integer writer consumed temporary storage");
    again, parse_error := parse_toml(text);

    assert(parse_error.kind == .NONE && toml_get(again.root, "x").integer == value);
    free_toml(*again);
    exact, exact_error := write_toml(doc.root, .{ max_output_bytes = text.count });

    assert(exact_error.kind == .NONE && exact == text);
    free(exact);
    _, short_error := write_toml(doc.root, .{ max_output_bytes = text.count - 1 });

    assert(short_error.kind == .LIMIT && toml_error_path(*short_error) == "$");
    free(text);
}

main :: () {
    doc, error := parse_toml("x=0");

    assert(error.kind == .NONE);
    defer free_toml(*doc);
    for s64.[0, 1, -1, 9223372036854775807, -9223372036854775808] check_integer(*doc, it);

    power: s64 = 10;

    for 1..18 {
        check_integer(*doc, power - 1);
        check_integer(*doc, power);
        check_integer(*doc, power + 1);
        check_integer(*doc, -power + 1);
        check_integer(*doc, -power);
        check_integer(*doc, -power - 1);
        if it < 18 power *= 10;
    }

    print("ok\n");
}
