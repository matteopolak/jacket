#import "Basic";
#import "String";
#import "Extensions/Toml";

main :: () {
    doc, error := new_toml_document();

    assert(error.kind == .NONE);
    value, value_error := toml_new_value(*doc, .FLOAT);

    assert(value_error.kind == .NONE);
    value.real = cast,force(float64) cast(u64) 0x8000000000000000;
    assert(toml_insert(*doc, doc.root, "negative-zero", value).kind == .NONE);
    text, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE && contains(text, "-0.0"), text);
    free(text);
    again, parse_error := parse_toml(text = "value = -0.0\n");

    assert(parse_error.kind == .NONE);
    assert((cast,force(u64) toml_get(again.root, "value").real) == 0x8000000000000000);
    free_toml(*again);
    assert(toml_insert(*doc, doc.root, "again", value).kind == .OUTPUT);

    temporal, temporal_error := toml_new_value(*doc, .LOCAL_DATE);

    assert(temporal_error.kind == .NONE);
    temporal.temporal = .{ year = 2024, month = 2, day = 30 };
    assert(toml_insert(*doc, doc.root, "invalid-date", temporal).kind == .NONE);
    _, invalid_date := write_toml(doc.root);

    assert(invalid_date.kind == .OUTPUT);
    assert(toml_error_path(*invalid_date) == "$[\"invalid-date\"]", toml_error_path(*invalid_date));
    temporal.temporal.day = 29;
    good, good_error := write_toml(doc.root);

    assert(good_error.kind == .NONE, good_error.message);
    free(good);
    free_toml(*doc);

    root: Toml_Value;
    first: Toml_Value;
    first.kind = .INTEGER;
    second: Toml_Value;
    second.kind = .INTEGER;
    array_add(*root.members, Toml_Member.{ key = "duplicate", value = *first });
    array_add(*root.members, Toml_Member.{ key = "duplicate", value = *second });
    _, duplicate := write_toml(*root);

    assert(duplicate.kind == .OUTPUT);
    array_free(root.members);
    check_print_style();
    print("ok\n");
}

check_print_style :: () {
    style := context.print_style;

    defer context.print_style = style;
    context.print_style.default_format_int.base = 16;
    context.print_style.default_format_float.decimal_separator_character = #char ",";
    doc, _ := new_toml_document();

    defer free_toml(*doc);
    value, _ := toml_new_value(*doc, .INTEGER);
    value.integer = 255;
    assert(toml_insert(*doc, doc.root, "integer", value).kind == .NONE);
    date, _ := toml_new_value(*doc, .LOCAL_DATE);
    date.temporal = .{ year = 2024, month = 2, day = 29 };
    assert(toml_insert(*doc, doc.root, "date", date).kind == .NONE);
    text, error := write_toml(doc.root);

    assert(error.kind == .NONE);
    defer free(text);
    again, parse_error := parse_toml(text);

    assert(parse_error.kind == .NONE, "writer ignores ambient display formatting");
    defer free_toml(*again);
    assert(toml_get(again.root, "integer").integer == 255);
    assert(toml_get(again.root, "date").temporal.year == 2024);
}
