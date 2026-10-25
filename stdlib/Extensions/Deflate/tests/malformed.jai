// Corrupt and truncated input must fail cleanly: ok is false, nothing is leaked, nothing is read
// or written out of bounds (the interpreter checks arrays; the allocator tally checks leaks).
//
//     jaic stdlib/Extensions/Deflate/tests/malformed.jai -interp

#load "support/helpers.jai";

expect_failure :: (stream: [] u8, container: Deflate_Container, label: string, wanted := "") {
    ok, outstanding, error := inflate_tracked(stream, container);

    check(!ok, tprint("%: was accepted", label));
    check(outstanding == 0, tprint("%: leaked % blocks", label, outstanding));
    check(error.count > 0 || ok, tprint("%: no error message", label));

    if wanted.count && !ok {
        check(error == wanted, tprint("%: error was \"%\", wanted \"%\"", label, error, wanted));
    }

    // inflate_into must fail the same way and never write past its buffer.
    guard := NewArray(64, u8);
    for *guard it.* = 0xCD;

    window: [] u8;
    window.data = guard.data + 16;
    window.count = 32;

    _, ok2 := inflate_into(stream, window, container);

    check(!ok2, tprint("%: inflate_into accepted it", label));

    for 0..15 check(guard[it] == 0xCD && guard[48 + it] == 0xCD, tprint("%: inflate_into wrote outside its buffer", label));

    free(guard.data);
}

zlib_header :: (cmf: u8, flg_without_check: u8) -> [] u8 {
    flg := flg_without_check;
    flg += cast,trunc(u8) ((31 - ((cast(u32) cmf * 256 + cast(u32) flg) % 31)) % 31);

    result: [..] u8;

    array_add(*result, cmf);
    array_add(*result, flg);

    return result;
}

// A zlib stream of "hello" with a replaced header.
with_header :: (header: [] u8, label: string, wanted: string) {
    stream: [..] u8;
    for header array_add(*stream, it);
    for hex("cb 48 cd c9 c9 07 00 06 2c 02 15") array_add(*stream, it);

    expect_failure(stream, .ZLIB, label, wanted);
}

headers :: () {
    expect_failure(.[], .ZLIB, "empty zlib", "unexpected end of data");
    expect_failure(.[], .GZIP, "empty gzip", "unexpected end of data");
    expect_failure(.[], .RAW, "empty raw", "unexpected end of data");
    expect_failure(.[0x78], .ZLIB, "one byte of zlib", "unexpected end of data");

    with_header(hex("78 9d"), "bad FCHECK", "bad zlib header");
    with_header(zlib_header(0x79, 0), "method is not deflate", "unsupported zlib compression method");
    with_header(zlib_header(0x88, 0), "window size above 32 KiB", "bad zlib window size");
    with_header(zlib_header(0x78, 0x20), "preset dictionary", "zlib preset dictionaries are not supported");
}

streams_with_bad_checksums :: () {
    good := hex("78 9c cb 48 cd c9 c9 07 00 06 2c 02 15");

    for 0..3 {
        copy := NewArray(good.count, u8);
        memcpy(copy.data, good.data, good.count);
        copy[good.count - 1 - it] ^= 1;

        expect_failure(copy, .ZLIB, tprint("adler32 byte % damaged", it), "adler32 checksum mismatch");
    }

    gzip := hex("1f 8b 08 00 00 00 00 00 00 ff cb 48 cd c9 c9 57 48 af ca 2c 00 00 19 6a d2 df 0a 00 00 00");

    for 0..7 {
        copy := NewArray(gzip.count, u8);
        memcpy(copy.data, gzip.data, gzip.count);
        copy[gzip.count - 1 - it] ^= 0x10;

        expect_failure(copy, .GZIP, tprint("gzip trailer byte % damaged", it));
    }

    for position: 0..3 {
        copy := NewArray(gzip.count, u8);

        memcpy(copy.data, gzip.data, gzip.count);

        if position == 0 copy[0] = 0x1E;
        else if position == 1 copy[2] = 7;
        else if position == 2 copy[3] = 0x20;
        else copy[3] = 0x80;

        expect_failure(copy, .GZIP, tprint("gzip header damaged %", position));
    }

    // FNAME with no terminator, FEXTRA longer than the data, FHCRC cut off.
    for flag: u8.[8, 4, 2, 16] {
        stream: [..] u8;
        for u8.[0x1F, 0x8B, 8] array_add(*stream, it);

        array_add(*stream, flag);
        for 0..5 array_add(*stream, 0);

        if flag == 4 {
            array_add(*stream, 200);
            array_add(*stream, 0);
        } else if flag == 8 || flag == 16 {
            for 0..15 array_add(*stream, #char "x");
        }

        expect_failure(stream, .GZIP, tprint("gzip optional field % cut off", flag));
    }
}

block_errors :: () {
    // Block type 3.
    expect_failure(hex("07"), .RAW, "block type 3", "invalid block type");

    // Stored blocks: NLEN is not the complement, data is cut off, header is cut off.
    expect_failure(hex("01 05 00 fa fe 68 65 6c 6c 6f"), .RAW, "stored length check", "stored block length check failed");
    expect_failure(hex("01 05 00 fa ff 68 65 6c"), .RAW, "stored data cut off", "unexpected end of data");
    expect_failure(hex("01 05 00 fa"), .RAW, "stored header cut off", "unexpected end of data");
    expect_failure(hex("00 00 00 ff ff"), .RAW, "stored block that is not the last");

    // Fixed blocks.
    a: Bit_Builder;

    put(*a, 1, 1);
    put(*a, 1, 2);
    put_code(*a, 0x30 + 97, 8);
    put_code(*a, 1, 7); // length 3
    put_code(*a, 30, 5); // distance symbol 30 does not exist
    put_code(*a, 0, 7);
    expect_failure(finish(*a), .RAW, "distance symbol 30", "invalid distance code");

    b: Bit_Builder;

    put(*b, 1, 1);
    put(*b, 1, 2);
    put_code(*b, 0xC0 + 6, 8); // literal/length symbol 286
    expect_failure(finish(*b), .RAW, "length symbol 286", "invalid literal/length code");

    c: Bit_Builder;

    put(*c, 1, 1);
    put(*c, 1, 2);
    put_code(*c, 0x30 + 97, 8);
    put_code(*c, 1, 7); // length 3
    put_code(*c, 1, 5); // distance 2, but only one byte exists
    put_code(*c, 0, 7);
    expect_failure(finish(*c), .RAW, "distance past the start", "distance is before the start of the output");

    d: Bit_Builder;

    put(*d, 1, 1);
    put(*d, 1, 2);
    put_code(*d, 1, 7); // a match first
    put_code(*d, 0, 5);
    put_code(*d, 0, 7);
    expect_failure(finish(*d), .RAW, "match with nothing before it");

    // No end-of-block code before the data ends.
    e: Bit_Builder;

    put(*e, 1, 1);
    put(*e, 1, 2);
    for 0..9 put_code(*e, 0x30 + 97, 8);

    expect_failure(finish(*e), .RAW, "no end of block", "unexpected end of data");
}

// A dynamic block whose literal/length code has the given lengths (symbols 0..count-1), with a
// valid single-code distance table.
dynamic_with :: (literal_lengths: [] u8, distance_lengths: [] u8) -> [] u8 {
    b: Bit_Builder;

    put_dynamic_header(*b, true, literal_lengths, distance_lengths);

    // Some body bytes; the header is what is under test.
    for 0..7 put(*b, 0, 8);

    return finish(*b);
}

code_errors :: () {
    lits := NewArray(259, u8);

    // Over-subscribed: three codes of length 1.
    lits[97] = 1;
    lits[98] = 1;
    lits[256] = 1;
    expect_failure(dynamic_with(lits, u8.[1]), .RAW, "over-subscribed literal code", "invalid literal/length code lengths");

    // Incomplete: two codes of length 2.
    for *lits it.* = 0;

    lits[97] = 2;
    lits[256] = 2;
    expect_failure(dynamic_with(lits, u8.[1]), .RAW, "incomplete literal code", "invalid literal/length code lengths");

    // No end-of-block code.
    for *lits it.* = 0;

    lits[97] = 1;
    lits[98] = 1;
    expect_failure(dynamic_with(lits, u8.[1]), .RAW, "no end-of-block code", "missing end-of-block code");

    // A complete literal code with bad distance codes.
    for *lits it.* = 0;

    lits[97] = 2;
    lits[256] = 2;
    lits[257] = 2;
    lits[258] = 2;
    expect_failure(dynamic_with(lits, u8.[1, 1, 1]), .RAW, "over-subscribed distance code", "invalid distance code lengths");
    expect_failure(dynamic_with(lits, u8.[2, 2]), .RAW, "incomplete distance code", "invalid distance code lengths");
    expect_failure(dynamic_with(lits, u8.[1, 0, 0, 0, 3]), .RAW, "incomplete distance code with gaps", "invalid distance code lengths");

    // A match with no distance codes defined.
    b: Bit_Builder;

    put_dynamic_header(*b, true, lits, u8.[0]);
    codes := canonical_codes(lits);

    put_code(*b, codes[97], 2);
    put_code(*b, codes[257], 2);
    put(*b, 0, 8);
    expect_failure(finish(*b), .RAW, "match without distance codes", "invalid distance code");

    // Too many literal/length or distance codes in the header.
    big := NewArray(288, u8);

    expect_failure(dynamic_with(big, u8.[1]), .RAW, "288 literal/length codes", "too many length or distance codes");

    far := NewArray(32, u8);

    expect_failure(dynamic_with(lits, far), .RAW, "32 distance codes", "too many length or distance codes");

    // Code length code: over-subscribed, then incomplete.
    c: Bit_Builder;

    put_header_start(*c, 257, 1, 4);
    put(*c, 1, 3);
    put(*c, 1, 3);
    put(*c, 1, 3);
    put(*c, 0, 3);
    for 0..7 put(*c, 0, 8);

    expect_failure(finish(*c), .RAW, "over-subscribed code length code", "over-subscribed code length code");

    d: Bit_Builder;

    put_header_start(*d, 257, 1, 4);
    put(*d, 2, 3);
    put(*d, 2, 3);
    put(*d, 0, 3);
    put(*d, 0, 3);
    for 0..7 put(*d, 0, 8);

    expect_failure(finish(*d), .RAW, "incomplete code length code", "incomplete code length code");

    // Repeat of the previous length with no previous length: symbols 0 and 16 share the code.
    e: Bit_Builder;

    put_header_start(*e, 257, 1, 4);
    put(*e, 1, 3);
    put(*e, 0, 3);
    put(*e, 0, 3);
    put(*e, 1, 3); // 16 and 0 have length 1
    put_code(*e, 1, 1); // symbol 16, first
    put(*e, 0, 2);
    for 0..7 put(*e, 0, 8);

    expect_failure(finish(*e), .RAW, "repeat with no previous length", "repeat with no previous length");

    // Zero runs that go past the number of lengths: symbol 18 repeated.
    f: Bit_Builder;

    put_header_start(*f, 257, 1, 4);
    put(*f, 0, 3);
    put(*f, 0, 3);
    put(*f, 1, 3);
    put(*f, 1, 3); // 18 and 0 have length 1

    for 0..3 {
        put_code(*f, 1, 1);
        put(*f, 127, 7);
    }

    for 0..7 put(*f, 0, 8);

    expect_failure(finish(*f), .RAW, "code lengths run past the table", "code lengths overflow");
}

// Valid streams of each kind, cut after every byte, and with every byte changed.
sweeps :: () {
    text := text_bytes(3000, 11);
    random := random_bytes(900, 12);

    inputs: [3][] u8;
    inputs[0] = text;
    inputs[1] = random;
    inputs[2] = bytes_of("x");

    for data: inputs {
        for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
            for level: s32.[0, 1, 6] {
                stream := deflate(data, container, level);

                // Every proper prefix is incomplete. The step keeps the interpreter run short.
                step := max(1, stream.count / 300);

                for length: 0..stream.count - 1 {
                    if length % step != 0 && length < stream.count - 12 continue;

                    prefix: [] u8;
                    prefix.data = stream.data;
                    prefix.count = length;

                    ok, outstanding := inflate_tracked(prefix, container);

                    if ok && container != .RAW {
                        check(false, tprint("% bytes of % accepted", length, container));
                        break;
                    }

                    // A raw stream may be complete before its end only if all blocks closed;
                    // such a prefix is never shorter than the whole stream here.
                    if ok && container == .RAW && length < stream.count {
                        check(false, tprint("raw prefix of % bytes accepted", length));
                        break;
                    }

                    if outstanding != 0 {
                        check(false, tprint("prefix of % bytes leaked", length));
                        break;
                    }
                }

                // Every byte changed in turn: any result is fine, but nothing may crash or leak.
                changed := NewArray(stream.count, u8);

                for position: 0..stream.count - 1 {
                    if position % step != 0 continue;

                    for 0..stream.count - 1 changed[it] = stream[it];

                    changed[position] ^= 0x5A;

                    _, outstanding := inflate_tracked(changed, container);

                    if outstanding != 0 {
                        check(false, tprint("change at % leaked", position));
                        break;
                    }
                }

                free(changed.data);
                free(stream.data);
            }
        }
    }
}

// Random bytes are almost never a valid stream; whatever they are, inflate returns.
noise :: () {
    lcg := Lcg.{ state = 77 };

    for trial: 0..399 {
        size := 1 + next(*lcg) % 64;
        data := NewArray(size, u8);

        for *data it.* = cast,trunc(u8) next(*lcg);

        for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
            _, outstanding := inflate_tracked(data, container);

            if outstanding != 0 {
                check(false, tprint("noise trial % leaked", trial));
                break;
            }
        }

        free(data.data);
    }
}

main :: () {
    headers();
    streams_with_bad_checksums();
    block_errors();
    code_errors();
    sweeps();
    noise();

    assert(failures == 0);
    print("ok\n");
}
