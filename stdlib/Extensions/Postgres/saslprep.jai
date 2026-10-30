// SASLprep (RFC 4013): the normalization SCRAM applies to a password before using it, so that
// "I<U+00AD>X" and "U+2168" both mean "IX". It maps, normalizes (NFKC), then rejects prohibited
// characters and mixes of right-to-left and left-to-right text. Like libpq, a password that
// cannot be prepared (invalid UTF-8, a prohibited character) is used as it is.
//
// Normalization covers what NFKC does to one code point at a time (saslprep_data.jai, generated
// by tools/pg_conformance/gen_saslprep.py). A base character followed by combining marks is not
// recomposed, and unassigned code points are not rejected.

#load "saslprep_data.jai";

// The prepared password and true, or `password` and false when it cannot be prepared. ASCII
// passwords come back unchanged. The result is in temporary storage unless it is `password`.
//
// - password: the password as the user gave it.
saslprep :: (password: string) -> string, bool {
    ascii := true;

    for password if it >= 0x80 {
        ascii = false;
        break;
    }

    if ascii return password, true;

    codes: [..] u32;
    codes.allocator = temp;

    at := 0;

    while at < password.count {
        code, size := utf8_decode(password, at);
        if size == 0 return password, false;

        at += size;

        if is_non_ascii_space(code) code = 0x20;
        else if maps_to_nothing(code) continue;

        append_normalized(*codes, code);
    }

    for codes if is_prohibited(it) return password, false;
    if !bidi_allowed(codes) return password, false;

    out: [..] u8;
    out.allocator = temp;

    for codes utf8_append(*out, it);

    return cast(string) out, true;
}

#scope_module

utf8_decode :: (text: string, at: s64) -> code: u32, size: s64 {
    first := text[at];
    if first < 0x80 return first, 1;

    size: s64;
    code: u32;
    minimum: u32;

    if first >= 0xC2 && first <= 0xDF {
        size = 2;
        code = first & 0x1F;
        minimum = 0x80;
    } else if first >= 0xE0 && first <= 0xEF {
        size = 3;
        code = first & 0x0F;
        minimum = 0x800;
    } else if first >= 0xF0 && first <= 0xF4 {
        size = 4;
        code = first & 0x07;
        minimum = 0x10000;
    } else {
        return 0, 0;
    }

    if at + size > text.count return 0, 0;

    for 1..size - 1 {
        next := text[at + it];
        if (next & 0xC0) != 0x80 return 0, 0;

        code = (code << 6) | (next & 0x3F);
    }

    if code < minimum || code > 0x10FFFF || (code >= 0xD800 && code <= 0xDFFF) return 0, 0;

    return code, size;
}

utf8_append :: (out: *[..] u8, code: u32) {
    if code < 0x80 {
        array_add(out, cast(u8) code);
    } else if code < 0x800 {
        array_add(out, cast(u8) (0xC0 | (code >> 6)));
        array_add(out, cast(u8) (0x80 | (code & 0x3F)));
    } else if code < 0x10000 {
        array_add(out, cast(u8) (0xE0 | (code >> 12)));
        array_add(out, cast(u8) (0x80 | ((code >> 6) & 0x3F)));
        array_add(out, cast(u8) (0x80 | (code & 0x3F)));
    } else {
        array_add(out, cast(u8) (0xF0 | (code >> 18)));
        array_add(out, cast(u8) (0x80 | ((code >> 12) & 0x3F)));
        array_add(out, cast(u8) (0x80 | ((code >> 6) & 0x3F)));
        array_add(out, cast(u8) (0x80 | (code & 0x3F)));
    }
}

append_normalized :: (codes: *[..] u32, code: u32) {
    low := 0;
    high := SASLPREP_CODES.count - 1;

    while low <= high {
        middle := (low + high) / 2;
        found := SASLPREP_CODES[middle];

        if found == code {
            for cast(s64) SASLPREP_STARTS[middle]..cast(s64) SASLPREP_STARTS[middle + 1] - 1 array_add(codes, SASLPREP_CHARS[it]);
            return;
        }

        if found < code low = middle + 1;
        else high = middle - 1;
    }

    array_add(codes, code);
}

in_ranges :: (code: u32, ranges: [] u32) -> bool {
    i := 0;

    while i < ranges.count {
        if code >= ranges[i] && code <= ranges[i + 1] return true;
        i += 2;
    }

    return false;
}

// RFC 3454 table C.1.2.
is_non_ascii_space :: (code: u32) -> bool {
    return code == 0xA0 || code == 0x1680 || (code >= 0x2000 && code <= 0x200B) || code == 0x202F || code == 0x205F || code == 0x3000;
}

// RFC 3454 table B.1.
maps_to_nothing :: (code: u32) -> bool {
    return code == 0xAD || code == 0x34F || code == 0x1806 || (code >= 0x180B && code <= 0x180D) || (code >= 0x200B && code <= 0x200D) || code == 0x2060 || (code >= 0xFE00 && code <= 0xFE0F) || code == 0xFEFF;
}

// RFC 4013 section 2.3: tables C.2.1, C.2.2, C.3 to C.9.
is_prohibited :: (code: u32) -> bool {
    PROHIBITED :: u32.[
        0x0, 0x1F, 0x7F, 0x9F, 0x340, 0x341, 0x6DD, 0x6DD, 0x70F, 0x70F, 0x180E, 0x180E,
        0x200C, 0x200F, 0x2028, 0x202E, 0x2060, 0x2063, 0x206A, 0x206F, 0x2FF0, 0x2FFB,
        0xD800, 0xDFFF, 0xE000, 0xF8FF, 0xFDD0, 0xFDEF, 0xFEFF, 0xFEFF, 0xFFF9, 0xFFFD,
        0x1D173, 0x1D17A, 0xE0001, 0xE0001, 0xE0020, 0xE007F, 0xF0000, 0xFFFFD, 0x100000, 0x10FFFD,
    ];

    if in_ranges(code, PROHIBITED) return true;

    // The last two code points of every plane are not characters.
    return (code & 0xFFFE) == 0xFFFE;
}

// RFC 3454 table D.1: characters read right to left.
is_right_to_left :: (code: u32) -> bool {
    RIGHT_TO_LEFT :: u32.[
        0x5BE, 0x5BE, 0x5C0, 0x5C0, 0x5C3, 0x5C3, 0x5D0, 0x5EA, 0x5F0, 0x5F4, 0x61B, 0x61B,
        0x61F, 0x61F, 0x621, 0x63A, 0x640, 0x64A, 0x66D, 0x66F, 0x671, 0x6D5, 0x6DD, 0x6DD,
        0x6E5, 0x6E6, 0x6FA, 0x6FE, 0x700, 0x70D, 0x710, 0x710, 0x712, 0x72C, 0x780, 0x7A5,
        0x7B1, 0x7B1, 0x200F, 0x200F, 0xFB1D, 0xFBB1, 0xFBD3, 0xFD3D, 0xFD50, 0xFDFC, 0xFE70, 0xFEFC,
    ];

    return in_ranges(code, RIGHT_TO_LEFT);
}

// Table D.2 lists every left-to-right letter; these ranges stand in for it.
is_left_to_right :: (code: u32) -> bool {
    LEFT_TO_RIGHT :: u32.[
        0x41, 0x5A, 0x61, 0x7A, 0xAA, 0xAA, 0xB5, 0xB5, 0xBA, 0xBA, 0xC0, 0xD6, 0xD8, 0xF6,
        0xF8, 0x2B8, 0x2BB, 0x2C1, 0x370, 0x58F, 0x900, 0x1FFF, 0x200E, 0x200E, 0x2071, 0x2C7F,
        0x3005, 0x303C, 0x3041, 0xD7FF, 0xFF21, 0xFF3A, 0xFF41, 0xFF5A, 0xFF66, 0xFFDC,
        0x10000, 0x1FFFF, 0x20000, 0x2FFFF,
    ];

    return in_ranges(code, LEFT_TO_RIGHT);
}

// RFC 3454 section 6: text with right-to-left characters may hold no left-to-right ones, and
// must begin and end with a right-to-left character.
bidi_allowed :: (codes: [] u32) -> bool {
    has_right_to_left := false;

    for codes if is_right_to_left(it) has_right_to_left = true;

    if !has_right_to_left return true;

    for codes if is_left_to_right(it) return false;

    return is_right_to_left(codes[0]) && is_right_to_left(codes[codes.count - 1]);
}
