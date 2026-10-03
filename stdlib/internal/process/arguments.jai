break_command_into_strings :: (input: string) -> [..] string {
    result: [..] string;
    result.allocator = temp;
    cursor: s64;
    while cursor < input.count {
        while cursor < input.count && input[cursor] == 32 cursor += 1;
        if cursor == input.count break;

        start := cursor;

        if input[cursor] == 34 {
            cursor += 1;
            start = cursor;
            while cursor < input.count && input[cursor] != 34 cursor += 1;

            array_add(*result, string.{ cursor - start, input.data + start });
            if cursor < input.count cursor += 1;
        } else {
            while cursor < input.count && input[cursor] != 32 cursor += 1;
            array_add(*result, string.{ cursor - start, input.data + start });
        }
    }

    return result;
}

append_quoted :: (builder: *String_Builder, s: string) {
    quoted := s.count == 0;
    for byte: s if byte == 32 || byte == 9 || byte == 34 quoted = true;

    if !quoted {
        append(builder, s);
        return;
    }

    append(builder, cast(u8) 34);
    slashes: s64;

    for byte: s {
        if byte == 92 {
            slashes += 1;
            continue;
        }

        if byte == 34 {
            for 0..slashes * 2 append(builder, cast(u8) 92);
        } else for 0..slashes - 1 append(builder, cast(u8) 92);

        append(builder, byte);
        slashes = 0;
    }

    for 0..slashes * 2 - 1 append(builder, cast(u8) 92);

    append(builder, cast(u8) 34);
}

get_quoted_command_string :: (args: [] string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for arg, i: args {
        if i > 0 append(*builder, cast(u8) 32);
        append_quoted(*builder, arg);
    }

    return builder_to_string(*builder);
}
