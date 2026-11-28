// Strict UTF-8 decoding, encoding and validation that never allocate.
//
// ```jai
// Utf8 :: #import "Extensions/Utf8";
//
// text := "caf\xc3\xa9";
// assert(Utf8.is_valid(text));
//
// code, size := Utf8.decode(text, 3); // the character that starts at byte 3
// assert(code == 0xe9 && size == 2);
//
// buffer: [4] u8;
// count := Utf8.encode(0x20ac, buffer.data);
// assert(count == 3 && buffer[0] == 0xe2);
//
// code, size = Utf8.decode("\xed\xa0\x80", 0); // a surrogate is not a character
// assert(size == 0);
// ```
//
// "Strict" is the standard's definition: overlong forms, the surrogates U+D800 to U+DFFF, values
// past U+10FFFF and truncated sequences are all invalid.

// The character that starts at byte `at` of `text`, and its length in bytes. An invalid or cut-off
// sequence gives `0, 0`, so a caller decides whether to stop, replace or skip one byte. `at` must
// be inside `text`.
//
// - text: the bytes to read.
// - at: the index of the first byte of the character.
//
// ```jai
// Utf8 :: #import "Extensions/Utf8";
//
// code, size := Utf8.decode("a\xf0\x9f\x98\x80", 1);
// assert(code == 0x1f600 && size == 4);
//
// code, size = Utf8.decode("\xe2\x82", 0); // cut short
// assert(code == 0 && size == 0);
// ```
decode :: (text: string, at: s64) -> code: u32, size: s64 {
    first := cast(u32) text[at];
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
        next := cast(u32) text[at + it];
        if (next & 0xC0) != 0x80 return 0, 0;

        code = (code << 6) | (next & 0x3F);
    }

    if code < minimum || code > 0x10FFFF || (code >= 0xD800 && code <= 0xDFFF) return 0, 0;

    return code, size;
}

// Writes `code` as UTF-8 at `out` and returns the number of bytes, 1 to 4. The four bytes at
// `out` are the most it touches. `code` is not checked: a surrogate is written as the three bytes
// strict decoding rejects, and a value past U+10FFFF as four bytes that decode as nothing, so check
// with `is_scalar` first when the source is not trusted.
//
// - code: the character to write.
// - out: where to write; room for 4 bytes.
//
// ```jai
// Utf8 :: #import "Extensions/Utf8";
//
// buffer: [4] u8;
// assert(Utf8.encode(0x41, buffer.data) == 1 && buffer[0] == 0x41);
// assert(Utf8.encode(0x1f600, buffer.data) == 4 && buffer[3] == 0x80);
// ```
encode :: (code: u32, out: *u8) -> s64 {
    if code < 0x80 {
        out[0] = cast(u8) code;
        return 1;
    }

    if code < 0x800 {
        out[0] = cast(u8) (0xC0 | (code >> 6));
        out[1] = cast(u8) (0x80 | (code & 0x3F));
        return 2;
    }

    if code < 0x10000 {
        out[0] = cast(u8) (0xE0 | (code >> 12));
        out[1] = cast(u8) (0x80 | ((code >> 6) & 0x3F));
        out[2] = cast(u8) (0x80 | (code & 0x3F));
        return 3;
    }

    out[0] = cast(u8) (0xF0 | (code >> 18));
    out[1] = cast(u8) (0x80 | ((code >> 12) & 0x3F));
    out[2] = cast(u8) (0x80 | ((code >> 6) & 0x3F));
    out[3] = cast(u8) (0x80 | (code & 0x3F));

    return 4;
}

// Whether `code` is a Unicode scalar value: at most U+10FFFF and not a surrogate.
//
// ```jai
// Utf8 :: #import "Extensions/Utf8";
//
// assert(Utf8.is_scalar(0x1f600));
// assert(!Utf8.is_scalar(0xd800));
// assert(!Utf8.is_scalar(0x110000));
// ```
is_scalar :: inline (code: u32) -> bool {
    return code <= 0x10FFFF && (code < 0xD800 || code > 0xDFFF);
}

// Whether every byte of `bytes` belongs to a strictly valid UTF-8 sequence. ASCII runs are
// skipped eight bytes at a time.
//
// - bytes: the text to check.
//
// ```jai
// Utf8 :: #import "Extensions/Utf8";
//
// assert(Utf8.is_valid("na\xc3\xafve"));
// assert(!Utf8.is_valid("na\xefve")); // Latin-1, not UTF-8
// ```
is_valid :: (bytes: [] u8) -> bool {
    i: s64;
    // Skip the ASCII prefix in complete unaligned words, then use the strict scalar
    // validator for the tail and non-ASCII text. Short strings pay one prefix check.
    while bytes.count - i >= 8 {
        word: u64;
        memcpy(*word, bytes.data + i, 8);
        if (word & 0x8080808080808080) != 0 break;
        i += 8;
    }

    if bytes.count - i >= 4 {
        word: u32;
        memcpy(*word, bytes.data + i, 4);
        if (word & 0x80808080) == 0 i += 4;
    }

    while i < bytes.count {
        a := bytes[i];

        i += 1;
        if a < 128 continue;

        count: s64;
        low: u8 = 128;
        high: u8 = 191;

        if a >= 194 && a <= 223 count = 1;
        else if a >= 224 && a <= 239 {
            count = 2;
            if a == 224 low = 160;
            if a == 237 high = 159;
        } else if a >= 240 && a <= 244 {
            count = 3;
            if a == 240 low = 144;
            if a == 244 high = 143;
        } else return false;

        if bytes.count - i < count return false;
        if bytes[i] < low || bytes[i] > high return false;
        for j: 1..count - 1 if bytes[i + j] < 128 || bytes[i + j] > 191 return false;

        i += count;
    }

    return true;
}

// `is_valid` for a string.
is_valid :: inline (text: string) -> bool {
    return is_valid(cast([] u8) text);
}
