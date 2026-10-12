// Byte scanning for the parser: the character classes of RFC 9110 and the vector loops that
// find bytes a field must not hold. Each vector loop has a scalar twin that gives the same
// answer; `http_use_simd` selects (wasm always takes the scalar one).

#scope_module

simd_enabled :: inline () -> bool {
    #if OS == .WASM return false;
    else return http_use_simd;
}

// Character classes, one bit per class in a 256-entry table.
CLASS_TOKEN :: 1; // tchar, RFC 9110 5.6.2
CLASS_DIGIT :: 2;
CLASS_HEX :: 4;

make_char_classes :: () -> [256] u8 {
    table: [256] u8;

    for 0..255 {
        c := cast(u8) it;
        bits: u8;

        if (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") bits |= CLASS_TOKEN;
        if c >= #char "0" && c <= #char "9" bits |= CLASS_TOKEN | CLASS_DIGIT | CLASS_HEX;
        if (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F") bits |= CLASS_HEX;

        if c == #char "!" || c == #char "#" || c == #char "$" || c == #char "%" || c == #char "&" || c == #char "'" || c == #char "*" || c == #char "+" || c == #char "-" || c == #char "." || c == #char "^" || c == #char "_" || c == #char "`" || c == #char "|" || c == #char "~" {
            bits |= CLASS_TOKEN;
        }

        table[it] = bits;
    }

    return table;
}

char_classes :: #run make_char_classes();

is_token_char :: inline (c: u8) -> bool {
    return (char_classes[c] & CLASS_TOKEN) != 0;
}

is_digit :: inline (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_ows :: inline (c: u8) -> bool {
    return c == #char " " || c == 9;
}

hex_value :: inline (c: u8) -> int {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    return -1;
}

lower :: inline (c: u8) -> u8 {
    if c >= #char "A" && c <= #char "Z" return c + 32;

    return c;
}

// The index of the first `byte` in data[start, end), or -1.
find_byte :: (data: *u8, start: int, end: int, byte: u8) -> int {
    i := start;

    if simd_enabled() {
        while i + 16 <= end {
            found := to_bits(eq(load(u8x16, data + i), byte));

            if found != 0 return i + trailing_zeros(found);

            i += 16;
        }
    }

    while i < end {
        if data[i] == byte return i;

        i += 1;
    }

    return -1;
}

// The index of the first byte in data[start, end) that a field value may not hold, or -1. A
// value holds visible characters, SP, HTAB and obs-text (0x80 and up); NUL, other controls and
// DEL are refused.
find_bad_value_byte :: (data: *u8, start: int, end: int) -> int {
    i := start;

    if simd_enabled() {
        while i + 16 <= end {
            chunk := load(u8x16, data + i);
            bad := to_bits((lt(chunk, 0x20) & ne(chunk, 9)) | eq(chunk, 0x7f));

            if bad != 0 return i + trailing_zeros(bad);

            i += 16;
        }
    }

    while i < end {
        c := data[i];

        if (c < 0x20 && c != 9) || c == 0x7f return i;

        i += 1;
    }

    return -1;
}

// The index of the first byte in data[start, end) that a request target may not hold (SP,
// controls, DEL and everything from 0x80 up), or -1.
find_bad_target_byte :: (data: *u8, start: int, end: int) -> int {
    i := start;

    if simd_enabled() {
        while i + 16 <= end {
            chunk := load(u8x16, data + i);
            bad := to_bits(le(chunk, 0x20) | ge(chunk, 0x7f));

            if bad != 0 return i + trailing_zeros(bad);

            i += 16;
        }
    }

    while i < end {
        c := data[i];

        if c <= 0x20 || c >= 0x7f return i;

        i += 1;
    }

    return -1;
}

// Finds the end of the line that starts at data[start], which continues a line whose last byte
// so far was a CR when `after_cr`. Returns the index of the LF that ends it (-1 if the data ends
// first) and whether a CR or LF stands alone, which no line may hold.
find_line_end :: (data: [] u8, start: int, after_cr: bool) -> int, bool {
    i := start;

    if after_cr && i < data.count {
        if data[i] != #char "\n" return -1, true;

        return i, false;
    }

    base := data.data;

    while i < data.count {
        // The next CR or LF.
        hit := -1;

        if simd_enabled() {
            while i + 16 <= data.count {
                chunk := load(u8x16, base + i);
                found := to_bits(eq(chunk, #char "\n") | eq(chunk, #char "\r"));

                if found != 0 {
                    hit = i + trailing_zeros(found);

                    break;
                }

                i += 16;
            }
        }

        if hit < 0 {
            while i < data.count {
                if base[i] == #char "\n" || base[i] == #char "\r" {
                    hit = i;

                    break;
                }

                i += 1;
            }
        }

        if hit < 0 return -1, false;

        if base[hit] == #char "\n" return -1, true; // an LF without its CR

        if hit + 1 >= data.count return -1, false; // the CR is the last byte so far

        if base[hit + 1] != #char "\n" return -1, true;

        return hit + 1, false;
    }

    return -1, false;
}
