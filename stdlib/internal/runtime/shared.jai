// What the runtime and the freestanding runtime both define the same way. Loaded by each, so it uses their
// `write_string`.

// Fills an array of `num_elements` elements of `size` bytes by copying the first one into the rest.
__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc {
    for i: 1..num_elements - 1 memcpy(start + i * size, start, size);
}

// Returns the number of bytes before the first zero byte at `str`, or 0 for null.
c_style_strlen :: (str: *u8) -> s64 #no_context {
    if !str return 0;
    count: s64 = 0;
    while str[count] != 0 count += 1;
    return count;
}

// Returns a string viewing the zero-terminated bytes at `c_string`, without copying. Null gives an
// empty string.
to_string :: (c_string: *u8) -> string #no_context {
    return .{ c_style_strlen(c_string), c_string };
}

// Writes `n` in `base`, clamped to 2..16, with lowercase digits and no prefix.
write_nonnegative_number :: (n: u64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc {
    radix := base;
    if radix < 2 radix = 2;
    if radix > 16 radix = 16;

    buffer: [64] u8;
    cursor: s64 = 64;
    remaining := n;

    while true {
        cursor -= 1;
        digit := remaining % cast(u64) radix;

        buffer[cursor] = "0123456789abcdef"[cast(s64) digit];
        remaining /= cast(u64) radix;
        if remaining == 0 break;
    }

    write_string(string.{ 64 - cursor, buffer.data + cursor }, to_standard_error);
}

// Writes `n` in `base`, clamped to 2..16, with a leading `-` when negative.
write_number :: (n: s64, base := 10, to_standard_error := false) #no_context #no_abc #no_aoc {
    if n < 0 {
        write_string("-", to_standard_error);
        write_nonnegative_number(0 - cast,no_check(u64) n, base, to_standard_error);
    } else write_nonnegative_number(cast(u64) n, base, to_standard_error);
}

// The default `context.logger`: writes the message through `write_string`, as error output when it has
// `.ERROR`, adding a newline if it lacks one.
runtime_support_default_logger :: (message: string, data: *void, info: Log_Info) {
    to_error := (info.common_flags & .ERROR) != 0;

    if message.count && message[message.count - 1] != #char "\n" {
        write_strings(message, "\n", to_standard_error = to_error);
    } else {
        write_string(message, to_error);
    }
}
