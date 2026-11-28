#import "Basic";
#import "Extensions/Testing";
Utf8 :: #import "Extensions/Utf8";

// Every code point class encodes and decodes to the same value, with the length its range needs.
encode_and_decode_round_trip :: () {
    codes := u32.[0x0, 0x41, 0x7f, 0x80, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xffff, 0x10000, 0x10ffff];
    sizes := s64.[1, 1, 1, 2, 2, 3, 3, 3, 3, 4, 4];

    for codes {
        buffer: [4] u8;
        size := Utf8.encode(it, buffer.data);

        expect_equal(size, sizes[it_index], "size");

        code, decoded := Utf8.decode(string.{ size, buffer.data }, 0);

        expect_equal(code, it, "code");
        expect_equal(decoded, size, "decoded size");
    }
} @test

// Decoding starts at an offset and stops at one character.
decode_reads_from_an_offset :: () {
    text := "a\xc3\xa9\xe2\x82\xac\xf0\x9f\x98\x80z";

    code, size := Utf8.decode(text, 1);

    expect_equal(code, 0xe9);
    expect_equal(size, 2);

    code, size = Utf8.decode(text, 3);
    expect_equal(code, 0x20ac);
    expect_equal(size, 3);

    code, size = Utf8.decode(text, 6);
    expect_equal(code, 0x1f600);
    expect_equal(size, 4);

    code, size = Utf8.decode(text, 10);
    expect_equal(code, #char "z");
    expect_equal(size, 1);
} @test

// Strict: an invalid sequence has size 0 at its first byte, with nothing consumed.
decode_rejects_what_is_not_utf8 :: () {
    bad := string.[
        "\x80", // a continuation byte alone
        "\xc0\x80", // overlong NUL
        "\xc1\xbf", // overlong
        "\xe0\x80\x80", // overlong
        "\xe0\x9f\xbf", // overlong, just under U+0800
        "\xed\xa0\x80", // surrogate U+D800
        "\xed\xbf\xbf", // surrogate U+DFFF
        "\xf0\x8f\xbf\xbf", // overlong, just under U+10000
        "\xf4\x90\x80\x80", // past U+10FFFF
        "\xf5\x80\x80\x80", // a lead byte that is never used
        "\xff",
        "\xc3", // cut short
        "\xe2\x82",
        "\xf0\x9f\x98",
        "\xc3\x28", // not a continuation
    ];

    for bad {
        code, size := Utf8.decode(it, 0);

        expect_equal(size, 0, tprint("size of case %", it_index));
        expect_equal(code, 0, tprint("code of case %", it_index));
        expect(!Utf8.is_valid(it), tprint("is_valid of case %", it_index));
    }
} @test

is_valid_checks_whole_strings :: () {
    expect(Utf8.is_valid(""));
    expect(Utf8.is_valid("plain ascii, long enough to cross the word-at-a-time prefix"));
    expect(Utf8.is_valid("\xc3\xa9\xe2\x82\xac\xf0\x9f\x98\x80"));
    expect(Utf8.is_valid("\xf4\x8f\xbf\xbf"), "the last code point");
    expect(Utf8.is_valid("\xed\x9f\xbf"), "just under the surrogates");
    expect(Utf8.is_valid("\xee\x80\x80"), "just over the surrogates");

    // The bad byte after a long ASCII run is still found.
    expect(!Utf8.is_valid("0123456789abcdef0123456789abcdef\xc3"));
    expect(!Utf8.is_valid("0123456789abcdef0123456789abcdef\xed\xa0\x80"));
    expect(!Utf8.is_valid("\xe9"), "Latin-1 e acute");

    bytes := u8.[0x68, 0x69];

    expect(Utf8.is_valid(bytes));
} @test

encode_leaves_the_bytes_past_the_character_alone :: () {
    buffer := u8.[0xaa, 0xaa, 0xaa, 0xaa];

    expect_equal(Utf8.encode(0xe9, buffer.data), 2);
    expect_equal(buffer[2], 0xaa);
    expect_equal(buffer[3], 0xaa);
} @test
