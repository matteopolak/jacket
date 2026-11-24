// Corrupt, truncated and oversized streams fail with ok = false and no output, and inflate stops at the
// end of a gzip member.
#import "Basic";
D :: #import "Extensions/Deflate";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// A fresh copy of `source`, so a test can change bytes without touching the original.
copy_bytes :: (source: [] u8) -> [] u8 {
    copy := alloc(source.count);
    memcpy(copy, source.data, source.count);
    return .{ source.count, cast(*u8) copy };
}

main :: () {
    text := cast([] u8) "abcabcabcabcabcabc, the same words again and again: abcabcabc";
    packed := D.deflate(text, .ZLIB);

    // Truncated stream.
    truncated := packed;
    truncated.count = packed.count / 2;
    output, ok, error := D.inflate(truncated, .ZLIB);

    expect(!ok && output.count == 0, "truncated zlib stream is rejected");

    // Wrong Adler-32 trailer.
    bad_sum := copy_bytes(packed);
    bad_sum[bad_sum.count - 1] ^= 0xff;
    output, ok, error = D.inflate(bad_sum, .ZLIB);
    expect(!ok && output.count == 0, "zlib with a wrong checksum is rejected");

    // Wrong gzip magic.
    gzip := D.deflate(text, .GZIP);
    bad_magic := copy_bytes(gzip);
    bad_magic[0] = 0x00;
    output, ok, error = D.inflate(bad_magic, .GZIP);
    expect(!ok && output.count == 0, "gzip with a wrong magic is rejected");

    // A container that does not match the bytes.
    output, ok, error = D.inflate(packed, .GZIP);
    expect(!ok && output.count == 0, "zlib bytes read as gzip are rejected");

    // Output limit: 70 KB of repeated text does not fit in 1000 bytes.
    big := cast(*u8) alloc(70 * 1024);

    for i: 0..70 * 1024 - 1 {
        big[i] = cast,trunc(u8) (97 + i % 7);
    }

    big_text: [] u8 = .{ 70 * 1024, cast(*u8) big };
    big_packed := D.deflate(big_text, .ZLIB);

    output, ok, error = D.inflate(big_packed, .ZLIB, 1000);
    expect(!ok && output.count == 0, "output over max_output is rejected");
    output, ok, error = D.inflate(big_packed, .ZLIB, 70 * 1024);
    expect(ok && output.count == big_text.count, "output exactly at max_output is accepted");

    // inflate_into with a buffer that is too small.
    small: [10] u8;
    _, into_ok := D.inflate_into(packed, small, .ZLIB);

    expect(!into_ok, "inflate_into with a short buffer fails");

    // inflate_member: two gzip members back to back; the first is consumed exactly.
    first := D.deflate(cast([] u8) "first member", .GZIP);
    second := D.deflate(cast([] u8) "second member", .GZIP);
    joined := cast(*u8) alloc(first.count + second.count);

    memcpy(joined, first.data, first.count);
    memcpy(joined + first.count, second.data, second.count);
    both: [] u8 = .{ first.count + second.count, cast(*u8) joined };

    // inflate ignores bytes after a gzip member.
    trailing, trailing_ok, trailing_error := D.inflate(both, .GZIP);

    expect(trailing_ok && cast(string) trailing == "first member", tprint("gzip with a trailing member: %", trailing_error));

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
