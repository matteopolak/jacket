// Known checksums and streams made by other implementations, and hand-built blocks of every
// type (stored, fixed, dynamic with a lone distance code), in all three containers.
//
//     jaic stdlib/Extensions/Deflate/tests/vectors.jai -interp

#load "support/helpers.jai";

expect_inflate :: (stream: [] u8, container: Deflate_Container, expected: [] u8, label: string) {
    output, ok, error := inflate(stream, container);

    check(ok, tprint("%: failed: %", label, error));

    if ok {
        check(same_bytes(output, expected), tprint("%: wrong output", label));
        free(output.data);
    }
}

checksums :: () {
    check(crc32(bytes_of("123456789")) == 0xCBF43926, "crc32 check value");
    check(crc32(.[]) == 0, "crc32 of nothing");
    check(adler32(bytes_of("Wikipedia")) == 0x11E60398, "adler32 of Wikipedia");
    check(adler32(.[]) == 1, "adler32 of nothing");

    // Continuing over pieces gives the whole-input result, at every split.
    data := text_bytes(5000, 7);
    whole_crc := crc32(data);
    whole_adler := adler32(data);

    for split: 0..data.count - 1 {
        if split % 97 != 0 && split > 40 continue;

        first: [] u8;
        first.data = data.data;
        first.count = split;

        second: [] u8;
        second.data = data.data + split;
        second.count = data.count - split;

        if crc32(second, crc32(first)) != whole_crc {
            check(false, tprint("crc32 split at %", split));
            break;
        }

        if adler32(second, adler32(first)) != whole_adler {
            check(false, tprint("adler32 split at %", split));
            break;
        }
    }

    // A long run of 0xFF keeps both sums at their largest.
    ones := NewArray(70000, u8);
    for *ones it.* = 255;

    a: u32 = 1;
    b: u32 = 0;

    for ones {
        a = (a + it) % 65521;
        b = (b + a) % 65521;
    }

    check(adler32(ones) == (b << 16) | a, "adler32 of 70000 x 0xFF");

    // Unaligned starts for the word-at-a-time loops.
    for start: 0..8 {
        piece: [] u8;
        piece.data = data.data + start;
        piece.count = 1000;

        copy := NewArray(1000, u8);
        for 0..999 copy[it] = piece[it];

        check(crc32(piece) == crc32(copy) && adler32(piece) == adler32(copy), "checksums do not depend on alignment");
        free(copy.data);
    }
}

streams :: () {
    hello := hex("78 9c cb 48 cd c9 c9 07 00 06 2c 02 15");

    expect_inflate(hello, .ZLIB, bytes_of("hello"), "zlib hello");

    // The encoder produces the same bytes for "hello".
    ours := deflate(bytes_of("hello"), .ZLIB, 6);

    check(same_bytes(ours, hello), "deflate of hello matches the reference stream");

    stored := hex("78 01 01 07 00 f8 ff 73 74 6f 72 65 64 21 0b ef 02 b3");

    expect_inflate(stored, .ZLIB, bytes_of("stored!"), "zlib stored block");

    fixed := hex("78 da 4b 4c 4a 4e 44 45 00 41 7c 06 e5");

    expect_inflate(fixed, .ZLIB, bytes_of("abcabcabcabcabcabc"), "zlib fixed block with matches");

    // Dynamic Huffman block.
    sentence := "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. How vexingly quick daft zebras jump! Sphinx of black quartz, judge my vow. ";
    text: [..] u8;

    for 1..4 for sentence array_add(*text, it);

    dynamic := hex("78 da ed 8e 4b 12 c2 20 10 44 af d2 ee ad 9c c3 a5 55 e6 02 20 03 41 09 93 10 7e e1 f4 21 96 a7 b0 5c bf ee d7 3d 4e 84 35 d9 e7 1b 32 70 f1 d0 5c f1 4a f3 b2 81 33 05 c4 8e 9d 68 3b 14 9b 01 77 d1 73 f3 0e d9 43 c5 c6 09 da 66 ea a8 91 87 b3 6b e2 d0 bb 66 1b 70 e3 82 4c d5 7a e3 f6 af 5e 09 1d d1 48 06 b1 7d 06 2e 78 2c 93 f5 15 ac 21 dd 29 5e 93 08 b1 5d 3b 55 86 ce 99 cc 65 c0 f8 3f f8 eb 07 0f 25 48 e7 81");

    expect_inflate(dynamic, .ZLIB, text, "zlib dynamic block");

    // Literals only, raw.
    expect_inflate(hex("4b 4c 4c 4c 4c 4c 4c 4c 4c 4a 4a 4a 02 00"), .RAW, bytes_of("aaaaaaaabbbb"), "raw fixed block of literals");

    gzip := hex("1f 8b 08 00 00 00 00 00 00 ff cb 48 cd c9 c9 57 48 af ca 2c 00 00 19 6a d2 df 0a 00 00 00");

    expect_inflate(gzip, .GZIP, bytes_of("hello gzip"), "gzip member");

    // Bytes after the stream are ignored.
    extra: [..] u8;
    for gzip array_add(*extra, it);
    for 0..9 array_add(*extra, 0xAA);

    expect_inflate(extra, .GZIP, bytes_of("hello gzip"), "gzip with trailing bytes");
}

// The gzip header fields: FEXTRA, FNAME, FCOMMENT and FHCRC, alone and together.
gzip_headers :: () {
    body := deflate(bytes_of("header test"), .RAW, 6);
    payload := bytes_of("header test");

    for flags: 0..15 {
        header: [..] u8;
        for u8.[0x1F, 0x8B, 8] array_add(*header, it);

        // The four test bits select FHCRC, FEXTRA, FNAME and FCOMMENT (bits 1 to 4 of FLG).
        mapped: u8;
        if flags & 1 mapped |= 2;
        if flags & 2 mapped |= 4;
        if flags & 4 mapped |= 8;
        if flags & 8 mapped |= 16;

        array_add(*header, mapped);
        for 0..5 array_add(*header, 0);

        if mapped & 4 {
            array_add(*header, 5);
            array_add(*header, 0);
            for 0..4 array_add(*header, cast,trunc(u8) (0x10 + it));
        }

        if mapped & 8 {
            for bytes_of("file.txt") array_add(*header, it);
            array_add(*header, 0);
        }

        if mapped & 16 {
            for bytes_of("a comment") array_add(*header, it);
            array_add(*header, 0);
        }

        if mapped & 2 {
            crc := crc32(header);
            array_add(*header, cast,trunc(u8) crc);
            array_add(*header, cast,trunc(u8) (crc >> 8));
        }

        stream: [..] u8;
        for header array_add(*stream, it);
        for body array_add(*stream, it);

        crc := crc32(payload);
        size := cast(u32) payload.count;

        for 0..3 array_add(*stream, cast,trunc(u8) (crc >> cast(u32) (it * 8)));
        for 0..3 array_add(*stream, cast,trunc(u8) (size >> cast(u32) (it * 8)));

        expect_inflate(stream, .GZIP, payload, tprint("gzip header flags %", mapped));

        if mapped & 2 {
            damaged := NewArray(stream.count, u8);
            for 0..stream.count - 1 damaged[it] = stream[it];

            damaged[header.count - 1] ^= 0x40;

            _, ok := inflate(damaged, .GZIP);

            check(!ok, "a wrong header checksum is rejected");
        }
    }
}

// Blocks assembled bit by bit, including the legal incomplete distance code.
built_blocks :: () {
    // Dynamic block: literal 'a', end of block, length codes 257 and 258 (lengths 3 and 4), and a
    // single distance code of length 1 (distance 1). "a" then a 3-match then a 4-match.
    b: Bit_Builder;
    literal_lengths := NewArray(259, u8);
    literal_lengths[97] = 2;
    literal_lengths[256] = 2;
    literal_lengths[257] = 2;
    literal_lengths[258] = 2;

    put_dynamic_header(*b, true, literal_lengths, u8.[1]);

    codes := canonical_codes(literal_lengths);

    put_code(*b, codes[97], 2);
    put_code(*b, codes[257], 2);
    put(*b, 0, 1); // distance code 0, the only code
    put_code(*b, codes[258], 2);
    put(*b, 0, 1);
    put_code(*b, codes[256], 2);

    expect_inflate(finish(*b), .RAW, bytes_of("aaaaaaaa"), "dynamic block with one distance code");

    // The same block with no distance code at all (a zero length) and no matches.
    c: Bit_Builder;

    put_dynamic_header(*c, true, literal_lengths, u8.[0]);
    put_code(*c, codes[97], 2);
    put_code(*c, codes[97], 2);
    put_code(*c, codes[256], 2);

    expect_inflate(finish(*c), .RAW, bytes_of("aa"), "dynamic block without distance codes");

    // Several blocks in a row: stored, fixed, dynamic, with a match reaching back across blocks.
    d: Bit_Builder;

    put(*d, 0, 1);
    put(*d, 0, 2);
    put(*d, 0, 5); // pad to the byte boundary after the 3 header bits
    put(*d, 3, 16);
    put(*d, 0xFFFC, 16);
    for bytes_of("xyz") put(*d, it, 8);

    // Fixed block: match of length 3 at distance 3, then end of block.
    put(*d, 0, 1);
    put(*d, 1, 2);
    put_code(*d, 1, 7); // length symbol 257 is 0000001 in the fixed code
    put_code(*d, 2, 5); // distance symbol 2 is distance 3
    put_code(*d, 0, 7); // end of block
    put(*d, 1, 1);
    put(*d, 1, 2);
    put_code(*d, 0, 7);

    expect_inflate(finish(*d), .RAW, bytes_of("xyzxyz"), "stored, fixed and an empty final block");

    // An empty stored final block.
    expect_inflate(hex("01 00 00 ff ff"), .RAW, .[], "empty stored block");

    // A fixed block of all 256 literal values with the 9-bit codes.
    e: Bit_Builder;

    put(*e, 1, 1);
    put(*e, 1, 2);

    for 0..255 {
        if it < 144 put_code(*e, cast(u32) (0x30 + it), 8);
        else put_code(*e, cast(u32) (0x190 + it - 144), 9);
    }

    put_code(*e, 0, 7);

    values := all_byte_values();
    values.count = 256;

    expect_inflate(finish(*e), .RAW, values, "fixed block of every byte value");
}

main :: () {
    checksums();
    streams();
    gzip_headers();
    built_blocks();

    assert(failures == 0);
    print("ok\n");
}
