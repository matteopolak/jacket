// Malformed UTF-8 decodes to the payload bits of its bytes; strict decoding stops before a bad
// continuation byte. Surrogates encode like any other code point.
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
    .{ u8.[0xC3, 0x28], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 0 },
    .{ u8.[0xE2, 0x82, 0x28], true, .SOURCE_ILLEGAL, Unicode.UNI_REPLACEMENT_CHAR, 1 },
    .{ u8.[0xC3, 0x28], false, .CONVERSION_OK, 0xe8, 2 },
    .{ u8.[0xE2, 0x82, 0x28], false, .CONVERSION_OK, 0x20a8, 3 },
    .{ u8.[0xF0, 0x28, 0x8C, 0xBC], false, .CONVERSION_OK, 0x2833c, 4 },
    .{ u8.[0x80], false, .CONVERSION_OK, 0, 1 },
    .{ u8.[0x80], true, .CONVERSION_OK, 0, 1 },
    .{ u8.[0xC3, 0xA9], true, .CONVERSION_OK, 0xe9, 2 },
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
    .{ 0xd800, "\xED\xA0\x80" },
    .{ 0xd7ff, "\xED\x9F\xBF" },
    .{ 0x110000, "\xEF\xBF\xBD" },
];

encodes_surrogates_and_out_of_range_values :: (code: u32, want: string) {
    assert_that(Unicode.character_utf32_to_utf8(code,, temp), is(want));
} @test @cases(ENCODINGS)
