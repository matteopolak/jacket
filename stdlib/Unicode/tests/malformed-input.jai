// Malformed UTF-8 is reported in both modes as the replacement character with an illegal result, taking
// the lead byte and the good continuation bytes. Well-formed sequences with no character behind them (overlong
// forms, surrogates, values above U+10FFFF) give the replacement character and the whole sequence in both modes,
// as an illegal result when strict. Surrogates and out-of-range values encode as U+FFFD.
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
    .{ u8.[0xC0, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xC0, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xC1, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xC1, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 2 },
    .{ u8.[0xE0, 0x80, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xE0, 0x80, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xE0, 0x9F, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xE0, 0x9F, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xF0, 0x80, 0x80, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF0, 0x80, 0x80, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF0, 0x8F, 0xBF, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF0, 0x8F, 0xBF, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xED, 0xA0, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xED, 0xA0, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xED, 0xBF, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xED, 0xBF, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 3 },
    .{ u8.[0xF4, 0x90, 0x80, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF4, 0x90, 0x80, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF7, 0xBF, 0xBF, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF7, 0xBF, 0xBF, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 4 },
    .{ u8.[0xF8, 0x88, 0x80, 0x80, 0x80], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 5 },
    .{ u8.[0xF8, 0x88, 0x80, 0x80, 0x80], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 5 },
    .{ u8.[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF], false, .CONVERSION_OK, Unicode.UNI_REPLACEMENT_CHAR, 6 },
    .{ u8.[0xFD, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 6 },
    .{ u8.[0xED, 0x9F, 0xBF], false, .CONVERSION_OK, 0xd7ff, 3 },
    .{ u8.[0xEE, 0x80, 0x80], true, .CONVERSION_OK, 0xe000, 3 },
    .{ u8.[0xF4, 0x8F, 0xBF, 0xBF], false, .CONVERSION_OK, 0x10ffff, 4 },
    .{ u8.[0xF4, 0x8F, 0xBF, 0xBF], true, .CONVERSION_OK, 0x10ffff, 4 },
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

next_character_replaces_a_surrogate_and_takes_all_of_it :: () {
    text := "\xED\xA0\x80A";
    code, status := Unicode.utf8_next_character(*text);

    assert_that(code, is(Unicode.UNI_REPLACEMENT_CHAR));
    assert_that(status, is(Unicode.Unicode_Result.CONVERSION_OK));
    assert_that(text, is("A"));
} @test
