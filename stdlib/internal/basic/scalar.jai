assert_helper :: (message := "", args: ..Any, loc := #caller_location) -> bool #no_debug {
    return context.assertion_failed(loc, tprint(message, ..args));
}

#if ENABLE_ASSERT {
    assert :: (arg: bool, message := "", args: ..Any, loc := #caller_location) #no_debug #expand {
        if !arg assert_helper(message, ..args, loc = loc);
    }
} else {
    assert :: (#discard arg: bool, #discard message := "", #discard args: ..Any, #discard loc := #caller_location) #expand {}
}

// Writes the formatted text to standard output, or standard error when requested.
// Returns the number of bytes written. Formatting uses the same argument grammar
// and active print style as the string-builder interface.
//
// ```
// print("% + % = %\n", 1, 2, 1 + 2);
// ```
//
// - format_string: the text and insertion markers
// - args: the values to print, in order
// - to_standard_error: select standard error instead of standard output
print :: (format_string: string, args: ..Any, to_standard_error := false) -> bytes_printed: s64 {
    builder: String_Builder;
    defer free_buffers(*builder);
    if !print_to_builder(*builder, format_string, ..args) return 0;
    return write_builder(*builder, to_standard_error = to_standard_error);
}

is_integer_type :: ($T: Type) -> (is_integer: bool, is_signed: bool, bits: u8) {
    info := type_info(T);
    if info.type != .INTEGER return false, false, 0;
    integer := cast(*Type_Info_Integer) info;
    return true, integer.signed, cast(u8) (info.runtime_size * 8);
}

min :: (a: $T, b: T) -> T {
    if a < b return a;
    return b;
} @UseResult @Minimum

max :: (a: $T, b: T) -> T {
    if a > b return a;
    return b;
} @UseResult @Maximum

min :: inline (a: $T, x: ..T) -> T #modify {
    return IsScalar(T), "min requires a scalar type";
} {
    result := a;
    for value: x if value < result result = value;
    return result;
} @UseResult @Minimum

max :: inline (a: $T, x: ..T) -> T #modify {
    return IsScalar(T), "max requires a scalar type";
} {
    result := a;
    for value: x if result < value result = value;
    return result;
} @UseResult @Maximum

clamp :: (x: $T, a: T, b: T) -> T #modify {
    return IsScalar(T), "clamp requires a scalar type";
} {
    return min(max(x, a), b);
} @UseResult

swap :: inline (a: *$T, b: *T) {
    t := a.*;
    a.* = b.*;
    b.* = t;
}

to_upper :: (c: u8) -> u8 {
    if c >= 97 && c <= 122 return c - 32;
    return c;
} @UseResult

to_lower :: (c: u8) -> u8 {
    if c >= 65 && c <= 90 return c + 32;
    return c;
} @UseResult

is_alpha :: (c: u8) -> bool {
    x := to_lower(c);
    return x >= 97 && x <= 122;
} @UseResult

is_digit :: (c: u8) -> bool {
    return c >= 48 && c <= 57;
} @UseResult

is_alnum :: (c: u8) -> bool {
    return is_alpha(c) || is_digit(c) || c == 95;
} @UseResult

is_space :: (c: u8) -> bool {
    return c == 32 || (c >= 9 && c <= 13);
} @UseResult

is_print :: (c: u8) -> bool {
    return c >= 32 && c <= 126;
}

advance :: inline (s: *string, amount: s64 = 1) {
    s.* = advance(s.*, amount);
}

advance :: inline (s: string, amount: s64 = 1) -> string {
    assert(amount >= 0 && amount <= s.count);
    return .{ s.count - amount, s.data + amount };
}
