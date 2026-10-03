print_sequence_to_builder :: (builder: *String_Builder, args: ..Any) {
    style := formatter_style();

    for item: args {
        render_item(builder, item, true, false, style);
        if builder.failed return;
    }
}

print_bytes :: (a: *void, length: int, format_string := "% ") {
    assert(length >= 0 && (a != null || length == 0));
    bytes := cast(*u8) a;
    for index: 0..length - 1 print(format_string, formatInt(bytes[index], base = 16));
    print("\n");
}

consume_u8_and_length :: (input: *string, s: *u8, length: s64) -> bool {
    assert(input != null);
    if length < 0 || length > input.count return false;
    if length == 0 return true;

    assert(s != null);
    memcpy(s, input.data, length);

    input.* = advance(input.*, length);

    return true;
}
