// How decoding one character went.
//
// `CONVERSION_OK` means a character was decoded. `SOURCE_EXHAUSTED` means the input ended in the middle
// of a character. `SOURCE_ILLEGAL` means the bytes are not valid UTF-8. Both decoding modes report it for a
// broken byte structure; strict decoding also reports it for a decoded value that is not a Unicode scalar.
Unicode_Result :: enum {
    CONVERSION_OK;
    SOURCE_EXHAUSTED;
    SOURCE_ILLEGAL;
}

// U+FFFD, the code point that stands in for a character that could not be decoded or encoded.
UNI_REPLACEMENT_CHAR :: 0x0000fffd;
// The highest code point of the Basic Multilingual Plane, U+FFFF.
UNI_MAX_BMP :: 0x0000ffff;
// The highest code point Unicode defines, U+10FFFF. Strict decoding and encoding reject anything above it.
UNI_MAX_UTF16 :: 0x0010ffff;
// The highest value non-strict decoding returns, 0x7FFFFFFF. A larger value decodes as `UNI_REPLACEMENT_CHAR`.
UNI_MAX_UTF32 :: 0x7fffffff;
// For a sequence of n + 1 bytes, the amount that classic decoders subtract after adding up the bytes
// shifted by six bits each. A table for hand-written decoders; the procedures here do not need it.
offsetsFromUTF8: [6] u32 : .[0, 0x3080, 0xe2080, 0x3c82080, 0xfa082080, 0x82082080];
// The top bits of the first byte of a sequence, by sequence length. A table for hand-written encoders;
// the procedures here do not need it.
firstByteMark: [7] u8 : .[0, 0, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc];
// The bits of the first byte of a sequence that hold the code point, indexed by the number of continuation
// bytes that follow (0 to 5).
utf8_inital_byte_mask :: u8.[0x7f, 0x1f, 0x0f, 0x07, 0x03, 0x01];
// How many continuation bytes follow a first byte, for each byte value. It is 0 for ASCII and for continuation
// bytes themselves, 1 to 3 for the lead bytes of valid sequences and 4 or 5 for the bytes of the old
// five and six byte forms (`0xF8` and up).
//
// ```jai
// assert(trailingBytesForUTF8[#char "a"] == 0);
// assert(trailingBytesForUTF8[0xc3] == 1);
// assert(trailingBytesForUTF8[0xe2] == 2);
// assert(trailingBytesForUTF8[0xf0] == 3);
// ```
trailingBytesForUTF8 :: #run trailing_table();

// Decodes the character that starts at `s`, where `source_length` bytes are available. Returns the code
// point, the number of bytes it took and how it went. A call with bytes available always takes at least one.
//
// Input that ends early gives `UNI_REPLACEMENT_CHAR`, all `source_length` bytes consumed and
// `SOURCE_EXHAUSTED`; no bytes at all gives 0 consumed.
//
// A broken byte structure is an error in both modes: a continuation byte (`0x80` to `0xBF`) where a character
// should start, or a byte that is not a continuation byte inside a character. The result is
// `UNI_REPLACEMENT_CHAR` with `SOURCE_ILLEGAL`, and the bytes consumed are the lead byte and the continuation
// bytes that were fine, so decoding can resume at the offending byte (a stray continuation byte takes just itself).
//
// Without `strict`, which is the default, a well-formed sequence is decoded as it is even when its value
// is overlong, a surrogate or above `UNI_MAX_UTF16`; values above `UNI_MAX_UTF32` become `UNI_REPLACEMENT_CHAR`
// with `CONVERSION_OK`. With `strict`, such a value gives `SOURCE_ILLEGAL` with the replacement character and
// the whole sequence consumed.
//
// ```jai
// code, size, status := character_utf8_to_utf32("\xe2\x82\xac".data, 3);
// assert(status == .CONVERSION_OK && code == 0x20ac && size == 3);
//
// bad := "\xc0\x80";
// code, size, status = character_utf8_to_utf32(bad.data, bad.count, true);
// assert(status == .SOURCE_ILLEGAL && code == UNI_REPLACEMENT_CHAR && size == 2);
//
// stray := "\x80";
// code, size, status = character_utf8_to_utf32(stray.data, stray.count);
// assert(status == .SOURCE_ILLEGAL && code == UNI_REPLACEMENT_CHAR && size == 1);
// ```
character_utf8_to_utf32 :: (s: *u8, source_length: s64, $strict := false) -> u32, s64, Unicode_Result #no_aoc {
    if source_length <= 0 return UNI_REPLACEMENT_CHAR, 0, .SOURCE_EXHAUSTED;

    lead := s[0];
    if lead < 0x80 return lead, 1, .CONVERSION_OK;
    if lead < 0xc0 return UNI_REPLACEMENT_CHAR, 1, .SOURCE_ILLEGAL;

    trailing := cast(s64) trailingBytesForUTF8[lead];
    length := trailing + 1;

    // Check the continuation bytes that are present before deciding the input is merely short.
    present := ifx length < source_length then length else source_length;

    for i: 1..present - 1 if (s[i] & 0xc0) != 0x80 return UNI_REPLACEMENT_CHAR, i, .SOURCE_ILLEGAL;

    if source_length < length return UNI_REPLACEMENT_CHAR, source_length, .SOURCE_EXHAUSTED;

    // The common lengths are spelled out so no loop runs for them.
    value := cast(u32) (lead & utf8_inital_byte_mask[trailing]);

    if trailing == 1 {
        value = (value << 6) | continuation(s[1]);
    } else if trailing == 2 {
        value = (value << 12) | (continuation(s[1]) << 6) | continuation(s[2]);
    } else if trailing == 3 {
        value = (value << 18) | (continuation(s[1]) << 12) | (continuation(s[2]) << 6) | continuation(s[3]);
    } else {
        for i: 1..trailing value = (value << 6) | continuation(s[i]);
    }

    #if strict {
        minimum: [6] u32 : .[0, 0x80, 0x800, 0x10000, 0x200000, 0x4000000];
        if value < minimum[trailing] || value > UNI_MAX_UTF16 || (value >= 0xd800 && value <= 0xdfff)
            return UNI_REPLACEMENT_CHAR, length, .SOURCE_ILLEGAL;
    }

    if value > UNI_MAX_UTF32 value = UNI_REPLACEMENT_CHAR;

    return value, length, .CONVERSION_OK;
}

// Decodes the first character of `s` and removes it from the start of `s`. Returns its code point and how
// decoding went, as `character_utf8_to_utf32` does without `strict`.
//
// At the end of the string the result is the replacement character and `SOURCE_EXHAUSTED` and `s` is
// not changed. A character cut short at the end of `s` also gives `SOURCE_EXHAUSTED` and takes all the bytes left.
// Malformed bytes give the replacement character and `SOURCE_ILLEGAL`, and at least one byte is removed.
//
// ```jai
// text := "a\xc3\xa9";
// code, status := utf8_next_character(*text);
// assert(status == .CONVERSION_OK && code == #char "a");
// code, status = utf8_next_character(*text);
// assert(status == .CONVERSION_OK && code == 0xe9 && text.count == 0);
// ```
utf8_next_character :: (s: *string) -> code: u32, success: Unicode_Result {
    code, count, status := character_utf8_to_utf32(s.data, s.count);

    // The decoder never reports more bytes than it was given.
    s.data += count;
    s.count -= count;

    return code, status;
}

// A pointer to the byte after the character that starts at `s`, going by its first byte only. Nothing is
// checked: a truncated or malformed character at the end of a buffer gives a pointer past the end.
unicode_next_character :: (s: *u8) -> *u8 {
    return s + 1 + cast(s64) trailingBytesForUTF8[s[0]];
}

// Writes the UTF-8 form of `ch` into the buffer `preallocated_string` points to, and sets its `count` to the
// number of bytes used (1 to 4). The buffer must have room for them, or an assert fails; four bytes always do.
//
// A value that is not a Unicode scalar value, which is a surrogate (U+D800 to U+DFFF) or anything above
// `UNI_MAX_UTF16`, is written as `UNI_REPLACEMENT_CHAR` (three bytes), so the output is always valid UTF-8.
//
// ```jai
// buffer: [4] u8;
// encoded := string.{ 4, buffer.data };
// character_utf32_to_utf8(0x20ac, *encoded);
// assert(encoded == "\xe2\x82\xac");
// ```
character_utf32_to_utf8 :: (ch: u32, preallocated_string: *string) {
    value := ch;
    data := preallocated_string.data;

    if value < 0x80 {
        assert(preallocated_string.count >= 1);
        data[0] = cast(u8) value;
        preallocated_string.count = 1;
        return;
    }

    if value > UNI_MAX_UTF16 || (value >= 0xd800 && value <= 0xdfff) value = UNI_REPLACEMENT_CHAR;

    if value < 0x800 {
        assert(preallocated_string.count >= 2);
        data[0] = cast(u8) (0xc0 | (value >> 6));
        data[1] = cast(u8) (0x80 | (value & 0x3f));
        preallocated_string.count = 2;
    } else if value < 0x10000 {
        assert(preallocated_string.count >= 3);
        data[0] = cast(u8) (0xe0 | (value >> 12));
        data[1] = cast(u8) (0x80 | ((value >> 6) & 0x3f));
        data[2] = cast(u8) (0x80 | (value & 0x3f));
        preallocated_string.count = 3;
    } else {
        assert(preallocated_string.count >= 4);
        data[0] = cast(u8) (0xf0 | (value >> 18));
        data[1] = cast(u8) (0x80 | ((value >> 12) & 0x3f));
        data[2] = cast(u8) (0x80 | ((value >> 6) & 0x3f));
        data[3] = cast(u8) (0x80 | (value & 0x3f));
        preallocated_string.count = 4;
    }
}

// The UTF-8 form of `ch`, in a new string. Values are encoded as in the other overload.
//
// The string is allocated with the context allocator and the caller frees it.
//
// ```jai
// encoded := character_utf32_to_utf8(0x1f642);
// defer free(encoded);
// assert(encoded.count == 4 && encoded == "\xf0\x9f\x99\x82");
// ```
character_utf32_to_utf8 :: (ch: u32) -> string {
    result := alloc_string(4);
    character_utf32_to_utf8(ch, *result);
    return result;
}

// Loops over the code points of a string: `for: utf8_iter text { ... }`. The body sees `it`, the code point,
// and `it_index`, the number of characters before it (not the byte offset). `for <:` goes from the last
// character to the first, and `it_index` is -1 there. Iterating by pointer is not allowed.
//
// The iteration consumes `text`: the string passed in is advanced as it goes. It is strict: on invalid UTF-8 (an
// overlong form, a surrogate, a bad or stray continuation byte or a character cut short) the loop logs `invalid UTF-8`
// as an error through the context logger and stops, so the rest of the string is left in `text` and the body
// does not see it. Going backwards, the last character and any stray bytes after it are checked first.
//
// ```jai
// text := "a\xc3\xa9\xe2\x82\xac";
// codes: [..] u32;
// defer array_free(codes);
// for: utf8_iter text array_add(*codes, it);
// assert(codes.count == 3 && codes[0] == #char "a" && codes[1] == 0xe9 && codes[2] == 0x20ac);
// ```
utf8_iter :: (s: *string, body: Code, flags: For_Flags) #expand {
    assert((flags & .POINTER) == 0);

    #if (flags & .REVERSE) != 0 {
        while s.count > 0 {
            index := s.count - 1;
            while index > 0 && (s.data[index] & 0xc0) == 0x80 index -= 1;

            code, consumed, status := character_utf8_to_utf32(s.data + index, s.count - index, true);
            if status != .CONVERSION_OK {
                report_invalid_utf8();
                break;
            }

            // Bytes after the decoded character are stray continuation bytes.
            if index + consumed < s.count {
                report_invalid_utf8();
                break;
            }

            s.count = index;
            `it := code;
            `it_index: s64 = -1;
            #insert body;
        }
    } else {
        index: s64;

        while s.count > 0 {
            code, consumed, status := character_utf8_to_utf32(s.data, s.count, true);
            if status != .CONVERSION_OK {
                report_invalid_utf8();
                break;
            }

            advance(s, consumed);
            `it := code;
            `it_index := index;
            defer index += 1;
            #insert body;
        }
    }
}

#scope_file
#import "Basic";

continuation :: inline (byte: u8) -> u32 {
    return cast(u32) (byte & 0x3f);
}

report_invalid_utf8 :: () {
    context.logger("invalid UTF-8\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

trailing_table :: () -> [256] u8 {
    result: [256] u8;
    for i: 0xc0..0xdf result[i] = 1;
    for i: 0xe0..0xef result[i] = 2;
    for i: 0xf0..0xf7 result[i] = 3;
    for i: 0xf8..0xfb result[i] = 4;
    for i: 0xfc..0xff result[i] = 5; // jailint: allow(magic_number): trailing byte count of the last lead-byte row, like the rows above

    return result;
}
