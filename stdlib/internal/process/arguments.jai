// Splits a command line into words: the pieces between spaces, where a piece in double quotes may hold spaces and loses its quotes.
//
// There are no escapes and no other separators. The words are slices of `input`, valid as long as it is, and the array is in
// temporary storage (`temp`), so neither is freed. An empty line gives no words; an unclosed quote runs to the end of the line.
//
// ```jai,skip_on(wasm)
// words := break_command_into_strings(" maple   \"red tree\" cedar ");
// assert(words.count == 3);
// assert(words[0] == "maple" && words[1] == "red tree" && words[2] == "cedar");
// ```
break_command_into_strings :: (input: string) -> [..] string {
    result: [..] string;
    result.allocator = temp;
    cursor: s64;
    while cursor < input.count {
        while cursor < input.count && input[cursor] == 32 cursor += 1;
        if cursor == input.count break;

        start := cursor;

        if input[cursor] == #char "\"" {
            cursor += 1;
            start = cursor;
            while cursor < input.count && input[cursor] != #char "\"" cursor += 1;

            array_add(*result, string.{ cursor - start, input.data + start });
            if cursor < input.count cursor += 1;
        } else {
            while cursor < input.count && input[cursor] != 32 cursor += 1;
            array_add(*result, string.{ cursor - start, input.data + start });
        }
    }

    return result;
}

// Appends `s` to `builder`, in double quotes when it has to be: when it is empty or holds a space, a tab or a double quote.
// A quote inside is escaped with a backslash, and backslashes before a quote or before the closing quote are doubled, which are
// the rules of the Windows command line, so the program at the other end splits it back into the same text.
//
// ```jai,skip_on(wasm)
// builder: String_Builder;
// append_quoted(*builder, "plain");
// append(*builder, " ");
// append_quoted(*builder, "two words");
// append(*builder, " ");
// append_quoted(*builder, "");
// text := builder_to_string(*builder);
// assert(text == "plain \"two words\" \"\"");
// free(text);
// ```
append_quoted :: (builder: *String_Builder, s: string) {
    quoted := s.count == 0;
    for byte: s if byte == 32 || byte == #char "\t" || byte == #char "\"" quoted = true;

    if !quoted {
        append(builder, s);
        return;
    }

    append(builder, cast(u8) 34);
    slashes: s64;

    for byte: s {
        if byte == #char "\\" {
            slashes += 1;
            continue;
        }

        if byte == #char "\"" {
            for 0..slashes * 2 append(builder, cast(u8) 92);
        } else for 0..slashes - 1 append(builder, cast(u8) 92);

        append(builder, byte);
        slashes = 0;
    }

    for 0..slashes * 2 - 1 append(builder, cast(u8) 92);

    append(builder, cast(u8) 34);
}

// Joins `args` into one command line with a space between them, each quoted as `append_quoted` does. The string is new, from
// `context.allocator`, and the caller frees it. Use it to show or log a command, or to build a Windows command line.
//
// ```jai,skip_on(wasm)
// line := get_quoted_command_string(.["plain", "two words", "say\"hi"]);
// assert(line == "plain \"two words\" \"say\\\"hi\"");
// free(line);
// ```
get_quoted_command_string :: (args: [] string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for arg, i: args {
        if i > 0 append(*builder, cast(u8) 32);
        append_quoted(*builder, arg);
    }

    return builder_to_string(*builder);
}
