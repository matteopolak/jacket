// CRC-32 and Adler-32 known answers, their chaining, and the header and trailer bytes of each container.
#import "Basic";
D :: #import "Extensions/Deflate";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

main :: () {
    check_text := cast([] u8) "123456789";

    expect(D.crc32(check_text) == 0xcbf43926, "crc32(\"123456789\")");
    expect(D.adler32(check_text) == 0x091e01de, "adler32(\"123456789\")");
    expect(D.crc32(.[]) == 0, "crc32 of empty input");
    expect(D.adler32(.[]) == 1, "adler32 of empty input");

    // Feeding the input in two pieces, each seeded with the previous result, gives the same checksum.
    crc_head := D.crc32(cast([] u8) "1234");

    expect(D.crc32(cast([] u8) "56789", crc_head) == 0xcbf43926, "crc32 chained over two pieces");
    adler_head := D.adler32(cast([] u8) "1234");

    expect(D.adler32(cast([] u8) "56789", adler_head) == 0x091e01de, "adler32 chained over two pieces");

    // ZLIB: header 0x78 0x9c (deflate, default level), then the Adler-32 of the input, big-endian.
    zlib := D.deflate(check_text, .ZLIB);

    expect(zlib[0] == 0x78 && zlib[1] == 0x9c, "zlib header bytes");
    n := zlib.count;

    expect(zlib[n - 4] == 0x09 && zlib[n - 3] == 0x1e && zlib[n - 2] == 0x01 && zlib[n - 1] == 0xde, "zlib trailer is Adler-32 big-endian");

    // GZIP: magic 0x1f 0x8b, method 8 (deflate), then CRC-32 and input size, both little-endian.
    gzip := D.deflate(check_text, .GZIP);

    expect(gzip[0] == 0x1f && gzip[1] == 0x8b && gzip[2] == 0x08, "gzip magic and method");
    n = gzip.count;
    expect(gzip[n - 8] == 0x26 && gzip[n - 7] == 0x39 && gzip[n - 6] == 0xf4 && gzip[n - 5] == 0xcb, "gzip trailer is CRC-32 little-endian");
    expect(gzip[n - 4] == 0x09 && gzip[n - 3] == 0 && gzip[n - 2] == 0 && gzip[n - 1] == 0, "gzip trailer is input size little-endian");

    // Empty input: ZLIB has the same 2-byte header and trailer for zero bytes of data.
    empty_zlib := D.deflate(.[], .ZLIB);
    output, ok, error := D.inflate(empty_zlib, .ZLIB);

    expect(ok && output.count == 0, tprint("empty zlib round trip: %", error));

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
