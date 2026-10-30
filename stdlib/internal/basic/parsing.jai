// Reads a whole number of type `T` (`int` by default) in the given `base` (2 to 36) from the start of `t`.
//
// Leading white space is skipped, then an optional `+` or `-`, then the digits; letters are digits
// above 9 and may be in either case. The result is the value, whether a number was read and the text
// after it. Reading stops at the first byte that is not a digit; it is not an error, so check the
// remainder if the whole text must be a number.
//
// When no digits are found, the value does not fit `T` or a minus sign is given for an unsigned type, the
// result is `0`, `false` and all of `t`. An unsupported `base` or a `T` that is not an integer fails an
// assert.
//
// ```jai
// value, ok, rest := string_to_int("  -42 apples");
// assert(ok && value == -42 && rest == " apples");
//
// hex, hex_ok, _ := string_to_int("ff", 16);
// assert(hex_ok && hex == 255);
//
// small, small_ok, _ := string_to_int("300", 10, u8);
// assert(!small_ok);
// ```
string_to_int :: (t: string, base := 10, $T := int) -> result: T, success: bool, remainder: string #no_aoc {
    MAX_BASE :: 36;
    info := cast(*Type_Info_Integer) type_info(T);

    assert(info.type == .INTEGER && base >= 2 && base <= MAX_BASE);
    cursor: s64;
    while cursor < t.count && is_space(t[cursor]) cursor += 1;

    negative := false;

    if cursor < t.count && (t[cursor] == #char "-" || t[cursor] == #char "+") {
        negative = t[cursor] == #char "-";
        cursor += 1;
    }

    start := cursor;
    limit: u64 = 0xffffffffffffffff;
    bits := size_of(T) * 8;

    if bits < 64 limit = ((cast(u64) 1) << bits) - 1;

    if info.signed {
        limit >>= 1;
        if negative limit += 1;
    } else if negative return 0, false, t;

    value: u64;

    while cursor < t.count {
        byte := to_lower(t[cursor]);
        digit: u64;

        if byte >= #char "0" && byte <= #char "9" digit = byte - #char "0";
        else if byte >= #char "a" && byte <= #char "z" digit = byte - #char "a" + 10;
        else break;
        if digit >= cast(u64) base break;
        if value > (limit - digit) / cast(u64) base return 0, false, t;

        value = value * cast(u64) base + digit;
        cursor += 1;
    }

    if cursor == start return 0, false, t;
    if negative value = 0 - value;

    return cast,trunc(T) value, true, advance(t, cursor);
} @UseResult

// Reads a decimal `int` from the start of `s`, as `string_to_int` does.
to_integer :: (s: string) -> result: int, success: bool, remainder: string {
    return string_to_int(s);
} @UseResult

// `_s` without its leading white space.
//
// ```jai
// assert(eat_spaces(" \t word") == "word");
// ```
eat_spaces :: (_s: string) -> string {
    i: s64;
    while i < _s.count && is_space(_s[i]) i += 1;
    return advance(_s, i);
}

// Reads a number into a `float64` from the start of `_s`: leading white space, an optional sign, then a decimal
// number with an optional fraction and exponent (`-1.5e3`), a hexadecimal one (`0x1.8p1`), `inf`,
// `infinity` or `nan`, in either case. The result is the value, whether a number was read and the text after it.
//
// When no number is found the result is `0`, `false` and all of `_s`. Reading stops at the first byte
// that cannot continue the number and is not an error, so check the remainder if the whole text must be a number.
//
// ```jai
// value, ok, rest := string_to_float64("2.5e2 units");
// assert(ok && value == 250 && rest == " units");
// ```
string_to_float64 :: (_s: string) -> (result: float64, success: bool, remainder: string) #no_abc {
    return parse_float_number(_s);
}

// Like `string_to_float64`, but the value is rounded to a `float`.
//
// ```jai
// value, ok, rest := string_to_float("1.25");
// assert(ok && value == 1.25 && rest.count == 0);
// ```
string_to_float :: (arg: string) -> (result: float, success: bool, remainder: string) {
    return parse_float_number(arg, float32);
} @UseResult

// Like `string_to_float64`, with the success flag first.
string_to_float64_new :: (_s: string) -> (success: bool, result: float64, remainder: string) {
    value, ok, rest := string_to_float64(_s);
    return ok, value, rest;
}

// Like `string_to_float`, with the success flag first.
string_to_float_new :: (arg: string) -> (success: bool, result: float, remainder: string) {
    value, ok, rest := string_to_float(arg);
    return ok, value, rest;
}
