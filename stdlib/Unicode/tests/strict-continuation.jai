// Strict decoding of a bad continuation byte consumes the lead byte, so a decode loop ends;
// surrogates are never encoded.
#import "Basic";
Unicode :: #import "Unicode";

decode_all :: (text: string) -> s64 {
    rest := text;
    steps: s64;

    while rest.count > 0 && steps < 100 {
        _, consumed, _ := Unicode.character_utf8_to_utf32(rest.data, rest.count, true);

        assert(consumed >= 1);
        advance(*rest, consumed);
        steps += 1;
    }

    assert(rest.count == 0);

    return steps;
}

main :: () {
    code, consumed, status := Unicode.character_utf8_to_utf32("\xC3\x28".data, 2, true);

    assert(status == .SOURCE_ILLEGAL && code == Unicode.UNI_REPLACEMENT_CHAR);
    assert(consumed == 1);
    assert(decode_all("\xC3\x28") == 2);
    assert(decode_all("\xE2\x82\x28") == 2);
    assert(decode_all("\xE2\x28\x28") == 3);
    assert(decode_all("\xF0\x90\x28A") == 3);

    code, consumed, status = Unicode.character_utf8_to_utf32("\xC3\xA9".data, 2, true);
    assert(status == .CONVERSION_OK && code == 0xe9 && consumed == 2);

    surrogate := Unicode.character_utf32_to_utf8(0xd800);

    assert(surrogate == "\xEF\xBF\xBD");
    surrogate = Unicode.character_utf32_to_utf8(0xdfff);
    assert(surrogate == "\xEF\xBF\xBD");
    ok := Unicode.character_utf32_to_utf8(0xd7ff);

    assert(ok == "\xED\x9F\xBF");
    ok = Unicode.character_utf32_to_utf8(0xe000);
    assert(ok == "\xEE\x80\x80");
    print("ok\n");
}
