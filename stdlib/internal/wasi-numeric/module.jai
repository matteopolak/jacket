// Independent C-locale wrappers around Basic's direct binary32/binary64 parser.
#scope_module
B :: #import "Basic";

libc :: #system_library "libc";
errno_location :: () -> *s32 #foreign libc "__errno_location";

c_text :: (text: *u8) -> string #no_context {
    length: s64;
    while text[length] != 0 length += 1;
    return .{ length, text };
}

space :: (c: u8) -> bool #no_context {
    return c == 32 || (c >= 9 && c <= 13);
}

nonzero_significand :: (text: string) -> bool #no_context {
    at: s64;
    while at < text.count && space(text[at]) at += 1;
    if at < text.count && (text[at] == 43 || text[at] == 45) at += 1;
    if at == text.count return false;

    first := text[at] | 32;
    if first == 105 || first == 110 return false; // inf/nan, not range errors.

    hex := at + 2 < text.count && text[at] == 48 && (text[at + 1] | 32) == 120;
    if hex at += 2;

    while at < text.count {
        c := text[at];
        lower := c | 32;

        if (!hex && lower == 101) || (hex && lower == 112) break;
        if (c >= 49 && c <= 57) || (hex && lower >= 97 && lower <= 102) return true;

        at += 1;
    }

    return false;
}

range_error :: (original: string, consumed: s64, value: $T) -> bool {
    bits: u64;
    infinity: u64;

    #if T == float32 {
        bits = cast(u64) (cast(*u32) *value).* & 0x7fffffff;
        infinity = 0x7f800000;
    } else {
        bits = (cast(*u64) *value).* & 0x7fffffffffffffff;
        infinity = 0x7ff0000000000000;
    }

    if bits != 0 && bits != infinity return false;

    // C11 7.22.1.3: underflow errno is implementation-defined. Signal
    // rounded-to-zero underflow; subnormal results preserve the prior errno.
    return nonzero_significand(.{ consumed, original.data });
}

#program_export "strtod"
wasi_strtod :: (text: *u8, end: **u8) -> float64 #c_call {
    local: #Context;

    push_context, defer_pop local;
    original := c_text(text);
    value, success, rest := B.string_to_float64(original);
    consumed := ifx success then original.count - rest.count else 0;

    if end end.* = text + consumed;
    if success && range_error(original, consumed, value) errno_location().* = 34;

    return value;
}

#program_export "strtof"
wasi_strtof :: (text: *u8, end: **u8) -> float32 #c_call {
    local: #Context;

    push_context, defer_pop local;
    original := c_text(text);
    value, success, rest := B.string_to_float(original);
    consumed := ifx success then original.count - rest.count else 0;

    if end end.* = text + consumed;
    if success && range_error(original, consumed, value) errno_location().* = 34;

    return value;
}

#program_export "atof"
wasi_atof :: (text: *u8) -> float64 #c_call {
    return wasi_strtod(text, null);
}

c_locale: [2] u8 = .[67, 0];

#program_export "setlocale"
wasi_setlocale :: (category: s32, name: *u8) -> *u8 #c_call {
    if category < 0 || category > 6 return null;
    if !name return c_locale.data;

    text := c_text(name);
    if text == "" || text == "C" || text == "POSIX" return c_locale.data;

    return null; // No unimplemented locale is reported as installed.
}
