assert_helper :: (message := "", args: ..Any, loc := #caller_location) -> bool #no_debug {
    return context.assertion_failed(loc, tprint(message, ..args));
}

#if ENABLE_ASSERT {
    assert :: (arg: bool, message := "", args: ..Any, loc := #caller_location) #no_debug #expand {
        if !arg assert_helper(message, ..args, loc = loc);
    } @AssertsArgument
} else {
    assert :: (#discard arg: bool, #discard message := "", #discard args: ..Any, #discard loc := #caller_location) #expand {} @AssertsArgument
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
} @PrintLike @WritesOutput

is_integer_type :: ($T: Type) -> (is_integer: bool, is_signed: bool, bits: u8) {
    info := type_info(T);
    if info.type != .INTEGER return false, false, 0;
    integer := cast(*Type_Info_Integer) info;
    return true, integer.signed, cast(u8) (info.runtime_size * 8);
}

min :: (a: $T, b: T) -> T {
    if a < b return a;
    return b;
} @UseResult @ReturnsSmaller

max :: (a: $T, b: T) -> T {
    if a > b return a;
    return b;
} @UseResult @ReturnsLarger

min :: inline (a: $T, x: ..T) -> T #modify {
    return IsScalar(T), "min requires a scalar type";
} {
    result := a;
    for value: x if value < result result = value;
    return result;
} @UseResult @ReturnsSmaller

max :: inline (a: $T, x: ..T) -> T #modify {
    return IsScalar(T), "max requires a scalar type";
} {
    result := a;
    for value: x if result < value result = value;
    return result;
} @UseResult @ReturnsLarger

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

// The capital for an ASCII lower case letter `c`; any other byte is returned as it is, so non-ASCII text is
// not changed.
//
// ```jai
// assert(to_upper(#char "a") == #char "A");
// assert(to_upper(#char "1") == #char "1");
// ```
to_upper :: (c: u8) -> u8 {
    if c >= #char "a" && c <= #char "z" return c - 32;
    return c;
} @UseResult

// The lower case letter for an ASCII capital `c`; any other byte is returned as it is.
//
// ```jai
// assert(to_lower(#char "Q") == #char "q");
// ```
to_lower :: (c: u8) -> u8 {
    if c >= #char "A" && c <= #char "Z" return c + 32;
    return c;
} @UseResult

// Whether `c` is an ASCII letter. Bytes of UTF-8 text above 127 are not letters.
//
// ```jai
// assert(is_alpha(#char "x") && !is_alpha(#char "7") && !is_alpha(#char "_"));
// ```
is_alpha :: (c: u8) -> bool {
    x := to_lower(c);
    return x >= #char "a" && x <= #char "z";
} @UseResult

// Whether `c` is an ASCII digit, `0` to `9`.
is_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
} @UseResult

// Whether `c` is an ASCII letter or digit, or an underscore: whether it can appear in an identifier.
//
// ```jai
// assert(is_alnum(#char "a") && is_alnum(#char "9") && is_alnum(#char "_"));
// assert(!is_alnum(#char "-"));
// ```
is_alnum :: (c: u8) -> bool {
    return is_alpha(c) || is_digit(c) || c == #char "_";
} @UseResult

// Whether `c` is white space: a space, tab, line feed, vertical tab, form feed or carriage return.
is_space :: (c: u8) -> bool {
    return c == #char " " || (c >= #char "\t" && c <= #char "\r");
} @UseResult

// Whether `c` is a printable ASCII character, from the space to `~`.
is_print :: (c: u8) -> bool {
    return c >= 32 && c <= #char "~";
}

// Removes the first `amount` bytes (one by default) from the string `s` points to. `amount` must not be
// negative or more than the length of the string, or an assert fails.
//
// ```jai
// text := "hello";
// advance(*text, 2);
// assert(text == "llo");
// ```
advance :: inline (s: *string, amount: s64 = 1) {
    s.* = advance(s.*, amount);
}

// The string `s` without its first `amount` bytes (one by default), pointing into the same memory.
// `amount` must not be negative or more than the length of `s`, or an assert fails.
//
// ```jai
// assert(advance("hello", 2) == "llo");
// ```
advance :: inline (s: string, amount: s64 = 1) -> string {
    assert(amount >= 0 && amount <= s.count);
    return .{ s.count - amount, s.data + amount };
}
