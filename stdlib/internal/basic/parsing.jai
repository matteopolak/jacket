string_to_int :: (t: string, base := 10, $T := int) -> result: T, success: bool, remainder: string #no_aoc {
    info := cast(*Type_Info_Integer) type_info(T);

    assert(info.type == .INTEGER && base >= 2 && base <= 36);
    cursor: s64;
    while cursor < t.count && is_space(t[cursor]) cursor += 1;

    negative := false;

    if cursor < t.count && (t[cursor] == 45 || t[cursor] == 43) {
        negative = t[cursor] == 45;
        cursor += 1;
    }

    start := cursor;
    limit: u64 = 0xffffffffffffffff;
    bits := size_of(T) * 8;

    if bits < 64 limit = (cast(u64) 1 << bits) - 1;

    if info.signed {
        limit >>= 1;
        if negative limit += 1;
    } else if negative return 0, false, t;

    value: u64;

    while cursor < t.count {
        byte := to_lower(t[cursor]);
        digit: u64;

        if byte >= 48 && byte <= 57 digit = byte - 48;
        else if byte >= 97 && byte <= 122 digit = byte - 97 + 10;
        else break;
        if digit >= cast(u64) base break;
        if value > (limit - digit) / cast(u64) base return 0, false, t;

        value = value * cast(u64) base + digit;
        cursor += 1;
    }

    if cursor == start return 0, false, t;
    if negative value = 0 - value;

    return cast,truncate(T) value, true, advance(t, cursor);
}

to_integer :: (s: string) -> result: int, success: bool, remainder: string {
    return string_to_int(s);
}

eat_spaces :: (_s: string) -> string {
    i: s64;
    while i < _s.count && is_space(_s[i]) i += 1;
    return advance(_s, i);
}

string_to_float64 :: (_s: string) -> (result: float64, success: bool, remainder: string) #no_abc {
    return parse_float_number(_s);
}

string_to_float :: (arg: string) -> (result: float, success: bool, remainder: string) {
    return parse_float_number(arg, float32);
}

string_to_float64_new :: (_s: string) -> (success: bool, result: float64, remainder: string) {
    value, ok, rest := string_to_float64(_s);
    return ok, value, rest;
}

string_to_float_new :: (arg: string) -> (success: bool, result: float, remainder: string) {
    value, ok, rest := string_to_float(arg);
    return ok, value, rest;
}
