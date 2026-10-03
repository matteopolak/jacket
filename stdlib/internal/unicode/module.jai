Unicode_Result :: enum {
    CONVERSION_OK;
    SOURCE_EXHAUSTED;
    SOURCE_ILLEGAL;
}

UNI_REPLACEMENT_CHAR :: 0x0000fffd;
UNI_MAX_BMP :: 0x0000ffff;
UNI_MAX_UTF16 :: 0x0010ffff;
UNI_MAX_UTF32 :: 0x7fffffff;
offsetsFromUTF8: [6] u32 : .[0, 0x3080, 0xe2080, 0x3c82080, 0xfa082080, 0x82082080];
firstByteMark: [7] u8 : .[0, 0, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc];
utf8_inital_byte_mask :: u8.[0x7f, 0x1f, 0x0f, 0x07, 0x03, 0x01];
trailingBytesForUTF8 :: #run trailing_table();

character_utf8_to_utf32 :: (s: *u8, source_length: s64, $strict := false) -> u32, s64, Unicode_Result #no_aoc {
    if source_length <= 0 return UNI_REPLACEMENT_CHAR, 0, .SOURCE_EXHAUSTED;

    trailing := cast(s64) trailingBytesForUTF8[s[0]];
    length := trailing + 1;

    if source_length < length return UNI_REPLACEMENT_CHAR, source_length, .SOURCE_EXHAUSTED;

    #if strict {
        if s[0] >= 0x80 && s[0] < 0xc0 return UNI_REPLACEMENT_CHAR, 1, .SOURCE_ILLEGAL;
        for i: 1..trailing if (s[i] & 0xc0) != 0x80 return UNI_REPLACEMENT_CHAR, i - 1, .SOURCE_ILLEGAL;
    }

    value: u32;
    for i: 0..trailing value = (value << 6) + cast(u32) s[i];

    value -= offsetsFromUTF8[trailing];

    #if strict {
        minimum: [6] u32 = .[0, 0x80, 0x800, 0x10000, 0x200000, 0x4000000];
        if value < minimum[trailing] || value > UNI_MAX_UTF16 || (value >= 0xd800 && value <= 0xdfff)
            return UNI_REPLACEMENT_CHAR, length, .SOURCE_ILLEGAL;
    }

    if value > UNI_MAX_UTF32 value = UNI_REPLACEMENT_CHAR;

    return value, length, .CONVERSION_OK;
}

utf8_next_character :: (s: *string) -> code: u32, success: Unicode_Result {
    code, count, status := character_utf8_to_utf32(s.data, s.count);
    advance(s, count);
    return code, status;
}

unicode_next_character :: (s: *u8) -> *u8 {
    return s + 1 + cast(s64) trailingBytesForUTF8[s[0]];
}

character_utf32_to_utf8 :: (ch: u32, preallocated_string: *string) {
    value := ch;
    if value > UNI_MAX_UTF16 value = UNI_REPLACEMENT_CHAR;

    count: s64 = 1;
    if value >= 0x10000 count = 4;
    else if value >= 0x800 count = 3;
    else if value >= 0x80 count = 2;

    assert(preallocated_string.count >= count);
    i := count - 1;
    while i > 0 {
        preallocated_string.data[i] = cast(u8) ((value & 63) | 0x80);
        value >>= 6;
        i -= 1;
    }

    preallocated_string.data[0] = cast(u8) value | firstByteMark[count];
    preallocated_string.count = count;
}

character_utf32_to_utf8 :: (ch: u32) -> string {
    result := alloc_string(4);
    character_utf32_to_utf8(ch, *result);
    return result;
}

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

report_invalid_utf8 :: () {
    context.logger("invalid UTF-8\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}

trailing_table :: () -> [256] u8 {
    result: [256] u8;
    for i: 0xc0..0xdf result[i] = 1;
    for i: 0xe0..0xef result[i] = 2;
    for i: 0xf0..0xf7 result[i] = 3;
    for i: 0xf8..0xfb result[i] = 4;
    for i: 0xfc..0xff result[i] = 5;

    return result;
}
