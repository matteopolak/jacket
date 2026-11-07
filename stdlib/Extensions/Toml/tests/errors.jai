#import "Basic";
#import "Extensions/Toml";

main :: () {
    _, error := parse_toml("[[records]]\nvalues = [1, { point = 1e+ }]\n");

    assert(error.kind == .SYNTAX);
    assert(toml_error_path(*error) == "$[\"records\"][0][\"values\"][1][\"point\"]", toml_error_path(*error));
    conflicts := string.["a=1\na.b=2\n", "a={}\na.b=2\n", "[a.b]\nx=1\n[a]\nb.y=2\n"];
    for conflicts {
        _, conflict := parse_toml(it);
        expected := ifx it_index == 2 then "$[\"a\"][\"b\"][\"y\"]" else "$[\"a\"][\"b\"]";
        assert(conflict.kind == .DUPLICATE && toml_error_path(*conflict) == expected, toml_error_path(*conflict));
    }

    _, unicode := parse_toml("\"é\" = 1\r\n\"é\" = 2\r\n");

    assert(unicode.kind == .DUPLICATE && unicode.line == 2 && unicode.column == 1);
    assert(unicode.span.start == 10 && unicode.related.start == 0);
    bytes := u8.[120, 32, 61, 32, 39, 0xc0, 0x80, 39];
    _, encoding := parse_toml(string.{ bytes.count, bytes.data });

    assert(encoding.kind == .ENCODING && encoding.span.start == 5);
    _, string_limit := parse_toml("x = 'ab'", .{ max_string_bytes = 1 });

    assert(string_limit.kind == .LIMIT);
    _, retained := parse_toml("x = 1", .{ max_retained_bytes = 1 });

    assert(retained.kind == .LIMIT);
    _, depth := parse_toml("[a.b]\nx = 1", .{ max_depth = 1 });

    assert(depth.kind == .LIMIT);
    _, fraction_without_seconds := parse_toml("time = 12:34.5");

    assert(fraction_without_seconds.kind == .SYNTAX);
    _, negative_limit := parse_toml("", .{ max_depth = -1 });

    assert(negative_limit.kind == .LIMIT);
    root: Toml_Value;
    _, negative_output := write_toml(*root, .{ max_output_bytes = -1 });

    assert(negative_output.kind == .LIMIT);
    print("ok\n");
}
