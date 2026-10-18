// Malformed UTF-8 decodes to the payload bits of its bytes; strict decoding stops before a bad
// continuation byte. Surrogates encode like any other code point.
#import "Basic";
Unicode :: #import "Unicode";

decode :: (bytes: [] u8, $strict := false) -> u32, s64, Unicode.Unicode_Result {
    code, consumed, status := Unicode.character_utf8_to_utf32(bytes.data, bytes.count, strict);
    return code, consumed, status;
}

main :: () {
    code, consumed, status := decode(u8.[0xC3, 0x28], true);

    assert(status == .SOURCE_ILLEGAL && code == Unicode.UNI_REPLACEMENT_CHAR && consumed == 0);
    code, consumed, status = decode(u8.[0xE2, 0x82, 0x28], true);
    assert(status == .SOURCE_ILLEGAL && consumed == 1);

    code, consumed, status = decode(u8.[0xC3, 0x28]);
    assert(status == .CONVERSION_OK && code == 0xe8 && consumed == 2);
    code, consumed, status = decode(u8.[0xE2, 0x82, 0x28]);
    assert(status == .CONVERSION_OK && code == 0x20a8 && consumed == 3);
    code, consumed, status = decode(u8.[0xF0, 0x28, 0x8C, 0xBC]);
    assert(status == .CONVERSION_OK && code == 0x2833c && consumed == 4);

    code, consumed, status = decode(u8.[0x80]);
    assert(status == .CONVERSION_OK && code == 0 && consumed == 1);
    code, consumed, status = decode(u8.[0x80], true);
    assert(status == .CONVERSION_OK && code == 0 && consumed == 1);

    code, consumed, status = decode(u8.[0xC3, 0xA9], true);
    assert(status == .CONVERSION_OK && code == 0xe9 && consumed == 2);

    assert(Unicode.character_utf32_to_utf8(0xd800) == "\xED\xA0\x80");
    assert(Unicode.character_utf32_to_utf8(0xd7ff) == "\xED\x9F\xBF");
    assert(Unicode.character_utf32_to_utf8(0x110000) == "\xEF\xBF\xBD");
    print("ok\n");
}
