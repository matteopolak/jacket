// Public CRC and Base64 vectors and the permissive decoder's edge cases.
#import "Basic";
#import "Extensions/Testing";
Base64 :: #import "Base64";
Crc :: #import "Crc";

Encoding :: struct {
    input: string;
    padded: string;
    unpadded: string;
}

ENCODINGS :: Encoding.[
    .{ "", "", "" },
    .{ "M", "TQ==", "TQ" },
    .{ "Ma", "TWE=", "TWE" },
    .{ "Man", "TWFu", "TWFu" },
    .{ "f", "Zg==", "Zg" },
    .{ "fo", "Zm8=", "Zm8" },
    .{ "foo", "Zm9v", "Zm9v" },
    .{ "foob", "Zm9vYg==", "Zm9vYg" },
    .{ "fooba", "Zm9vYmE=", "Zm9vYmE" },
    .{ "foobar", "Zm9vYmFy", "Zm9vYmFy" },
];

encodes :: (input: string, padded: string, unpadded: string) {
    assert_that(Base64.base64_encode(input,, temp), is(padded));
    assert_that(Base64.base64_encode(input, pad = false,, temp), is(unpadded));
} @test @cases(ENCODINGS)

round_trips :: (input: string, padded: string, unpadded: string) {
    for text: string.[padded, unpadded] {
        decoded, valid := Base64.base64_decode(text,, temp);
        expect(valid);
        expect_equal(decoded, input);
    }
} @test @cases(ENCODINGS)

Decoding :: struct {
    encoded: string;
    want: string;
    valid: bool;
}

// The decoder ignores what follows a `=` and line feeds, stops at a character outside the
// alphabet, and returns what it decoded up to there.
DECODINGS :: Decoding.[
    .{ "TWE", "Ma", true },
    .{ "T", "", true },
    .{ "TWE=ignored", "Ma", true },
    .{ "TQ=!\xff", "M", true },
    .{ "TQ\n==", "M", true },
    .{ "T!WE", "", false },
    .{ "TWE!rest", "Ma", false },
    .{ "TQ\r==", "M", false },
];

decodes_permissively :: (encoded: string, want: string, valid: bool) {
    decoded, ok := Base64.base64_decode(encoded,, temp);
    expect_equal(decoded, want);
    expect_equal(ok, valid);
} @test @cases(DECODINGS)

url_alphabet :: () {
    assert_that(Base64.base64url_encode("??",, temp), is("Pz8"));
    decoded, valid := Base64.base64url_decode("Pz8",, temp);
    expect(valid);
    assert_that(decoded, is("??"));
} @test

Checksum :: struct {
    input: string;
    want: u64;
}

CHECKSUMS :: Checksum.[
    .{ "", 0 },
    .{ "123456789", 0x62ec59e3f1a4f00a },
];

crc64_matches_the_published_check_values :: (input: string, want: u64) {
    assert_that(Crc.crc64(input), is(want));
} @test @cases(CHECKSUMS)

decoder_table_marks_line_feed_padding_and_garbage :: () {
    assert_that(Base64.BASE64_DECODER_TABLE[10], is(64));
    assert_that(Base64.BASE64_DECODER_TABLE[61], is(65));
    assert_that(Base64.BASE64_DECODER_TABLE[0], is(66));
} @test
