#import "Basic";
#import "Extensions/Toml";

main :: () {
    expected := "AZaz09_- = \"value\"\n123 = 2\ntrue = false\n\"\" = 0\n\"a.b\" = 1\n\"a b\" = 2\n\"é\" = 3\n\"a\\\"b\" = 4\n\"a\\\\b\" = 5\n\"a\\u0000b\" = 6\n";
    doc, parse_error := parse_toml(expected);

    assert(parse_error.kind == .NONE);
    defer free_toml(*doc);
    output, error := write_toml(doc.root);

    assert(error.kind == .NONE && output == expected, output);
    defer free(output);
    again, again_error := parse_toml(output);

    assert(again_error.kind == .NONE);
    defer free_toml(*again);
    assert(toml_get(again.root, "AZaz09_-").text == "value");
    assert(toml_get(again.root, "a\0b").integer == 6);

    // Bare-key emission must admit its complete bytes before copying any output.
    single, _ := parse_toml("abcd = 1");

    defer free_toml(*single);
    exact, exact_error := write_toml(single.root, .{ max_string_bytes = 4, max_output_bytes = 9 });

    assert(exact_error.kind == .NONE && exact == "abcd = 1\n");
    free(exact);
    limited, key_limit := write_toml(single.root, .{ max_string_bytes = 3 });

    assert(limited.count == 0 && key_limit.kind == .LIMIT && toml_error_path(*key_limit) == "$[\"abcd\"]");
    byte_limited, byte_limit := write_toml(single.root, .{ max_output_bytes = 3 });

    assert(byte_limited.count == 0 && byte_limit.kind == .LIMIT && toml_error_path(*byte_limit) == "$[\"abcd\"]");
    newline_limited, newline_limit := write_toml(single.root, .{ max_output_bytes = 8 });

    assert(newline_limited.count == 0 && newline_limit.kind == .LIMIT);

    // A non-ASCII invalid byte must take the checked quoted fallback, never a bare path.
    invalid: [1] u8;
    invalid[0] = 255;
    value: Toml_Value;
    value.kind = .INTEGER;
    raw: Toml_Value;

    array_add(*raw.members, Toml_Member.{ key = string.{ 1, invalid.data }, value = *value });
    defer array_free(raw.members);
    failed, invalid_error := write_toml(*raw);

    assert(failed.count == 0 && invalid_error.kind == .OUTPUT);
    no_output, invalid_before_output := write_toml(*raw, .{ max_output_bytes = 0 });

    assert(no_output.count == 0 && invalid_before_output.kind == .OUTPUT);
    no_key, limit_before_encoding := write_toml(*raw, .{ max_string_bytes = 0 });

    assert(no_key.count == 0 && limit_before_encoding.kind == .LIMIT);
}
