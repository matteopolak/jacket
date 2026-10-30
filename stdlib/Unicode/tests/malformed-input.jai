// Malformed UTF-8 is reported in both modes as the replacement character with an illegal result, taking
// the lead byte and the good continuation bytes. Surrogates and out-of-range values encode as U+FFFD.
#import "Basic";
#import "Extensions/Testing";
Unicode :: #import "Unicode";

Decoding :: struct {
    bytes: [] u8;
    strict: bool;
    status: Unicode.Unicode_Result;
    code: u32;
    consumed: s64;
}

DECODINGS :: Decoding.[
    .{ u8.[0xC3, 0x28], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xE2, 0x82, 0x28], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xC3, 0x28], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xE2, 0x82, 0x28], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xF0, 0x28, 0x8C, 0xBC], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xE2, 0x28], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0x80], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xBF, 0x41], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xBF, 0x41], false, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xC3, 0xA9], true, .CONVERSION_OK, 0xe9, 2 },
    .{ u8.[0xC0, 0x80], false, .CONVERSION_OK, 0, 2 },
    .{ u8.[0xC0, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xED, 0xA0, 0x80], false, .CONVERSION_OK, 0xd800, 3 },
    .{ u8.[0xED, 0xA0, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 3 },
];

decodes :: (bytes: [] u8, strict: bool, status: Unicode.Unicode_Result, code: u32, consumed: s64) {
    got_code, got_consumed, got_status := Unicode.character_utf8_to_utf32(bytes.data, bytes.count);
    if strict got_code, got_consumed, got_status = Unicode.character_utf8_to_utf32(bytes.data, bytes.count, true);

    assert_that(got_status, is(status));
    assert_that(got_consumed, is(consumed));

    assert_that(got_code, is(code));
} @test @cases(DECODINGS)

Encoding :: struct {
    code: u32;
    want: string;
}

ENCODINGS :: Encoding.[
    .{ 0xd800, "\xEF\xBF\xBD" },
    .{ 0xdfff, "\xEF\xBF\xBD" },
    .{ 0xd7ff, "\xED\x9F\xBF" },
    .{ 0xe000, "\xEE\x80\x80" },
    .{ 0x10ffff, "\xF4\x8F\xBF\xBF" },
    .{ 0x110000, "\xEF\xBF\xBD" },
    .{ 0xffffffff, "\xEF\xBF\xBD" },
];

encodes_surrogates_and_out_of_range_values_as_replacement :: (code: u32, want: string) {
    assert_that(Unicode.character_utf32_to_utf8(code,, temp), is(want));
} @test @cases(ENCODINGS)
