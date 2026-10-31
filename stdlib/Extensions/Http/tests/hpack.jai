// HPACK against the examples of RFC 7541 appendix C (requests and responses, with and without
// Huffman coding, with the dynamic table growing and evicting), the Huffman code on every byte,
// integers at their limits and the ways a block can be wrong.
//
//     jaic stdlib/Extensions/Http/tests/hpack.jai -interp

#load "support/support.jai";

hex_bytes :: (text: string) -> [] u8 {
    bytes: [..] u8;
    bytes.allocator = temp;

    digits := 0;
    high: u8;

    for text {
        if it == #char " " continue;

        digit: u8;

        if it >= #char "a" digit = cast(u8) (it - #char "a" + 10);
        else digit = cast(u8) (it - #char "0");

        if digits % 2 == 0 {
            high = digit;
        } else {
            array_add(*bytes, cast(u8) (high * 16 + digit));
        }

        digits += 1;
    }

    return bytes;
}

hex_of :: (bytes: [] u8) -> string {
    builder: String_Builder;

    for bytes print(*builder, "%0%", ifx it < 16 then "0" else "", FormatInt.{ value = it, base = 16 });

    return builder_to_string(*builder,, temp);
}

Field :: struct {
    name: string;
    value: string;
}

// Decodes `hex` and compares the fields (and the table, through the size it reports) with the example.
expect_block :: (decoder: *Hpack_Decoder, hex: string, fields: [] Field, table_size: int, what: string) {
    block: Hpack_Block;

    defer hpack_block_free(*block);

    error := hpack_decode(decoder, hex_bytes(hex), 1 << 20, *block);

    check(error == .NO_ERROR, tprint("% decodes", what));
    check(block.fields.count == fields.count, tprint("% has % fields, got %", what, fields.count, block.fields.count));

    if block.fields.count != fields.count return;

    for fields {
        check(block.fields[it_index].name == it.name && block.fields[it_index].value == it.value, tprint("% field %: %: % (got %: %)", what, it_index, it.name, it.value, block.fields[it_index].name, block.fields[it_index].value));
    }

    check(decoder.table.size == table_size, tprint("% leaves a table of % bytes, got %", what, table_size, decoder.table.size));
}

// Encodes the same fields and compares the bytes with the example.
expect_encoding :: (encoder: *Hpack_Encoder, fields: [] Field, hex: string, what: string) {
    out: [..] u8;

    defer array_reset(*out);

    hpack_encode_start(encoder, *out);

    for fields hpack_encode_field(encoder, *out, it.name, it.value);

    check(hex_of(out) == hex_of(hex_bytes(hex)), tprint("% encodes to % (got %)", what, hex_of(hex_bytes(hex)), hex_of(out)));
}

requests :: () {
    first := Field.[.{ ":method", "GET" }, .{ ":scheme", "http" }, .{ ":path", "/" }, .{ ":authority", "www.example.com" }];
    second := Field.[.{ ":method", "GET" }, .{ ":scheme", "http" }, .{ ":path", "/" }, .{ ":authority", "www.example.com" }, .{ "cache-control", "no-cache" }];
    third := Field.[.{ ":method", "GET" }, .{ ":scheme", "https" }, .{ ":path", "/index.html" }, .{ ":authority", "www.example.com" }, .{ "custom-key", "custom-value" }];

    // C.3: without Huffman coding.
    decoder: Hpack_Decoder;

    hpack_decoder_init(*decoder);
    defer hpack_decoder_free(*decoder);

    expect_block(*decoder, "8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d", first, 57, "C.3.1");
    expect_block(*decoder, "8286 84be 5808 6e6f 2d63 6163 6865", second, 110, "C.3.2");
    expect_block(*decoder, "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65", third, 164, "C.3.3");

    // C.4: with Huffman coding, decoded and encoded.
    huffman: Hpack_Decoder;

    hpack_decoder_init(*huffman);
    defer hpack_decoder_free(*huffman);

    expect_block(*huffman, "8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff", first, 57, "C.4.1");
    expect_block(*huffman, "8286 84be 5886 a8eb 1064 9cbf", second, 110, "C.4.2");
    expect_block(*huffman, "8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf", third, 164, "C.4.3");

    encoder: Hpack_Encoder;

    hpack_encoder_init(*encoder);
    defer hpack_encoder_free(*encoder);

    expect_encoding(*encoder, first, "8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff", "encoding C.4.1");
    expect_encoding(*encoder, second, "8286 84be 5886 a8eb 1064 9cbf", "encoding C.4.2");
    expect_encoding(*encoder, third, "8287 85bf 4088 25a8 49e9 5ba9 7d7f 8925 a849 e95b b8e8 b4bf", "encoding C.4.3");
}

responses :: () {
    first := Field.[.{ ":status", "302" }, .{ "cache-control", "private" }, .{ "date", "Mon, 21 Oct 2013 20:13:21 GMT" }, .{ "location", "https://www.example.com" }];
    second := Field.[.{ ":status", "307" }, .{ "cache-control", "private" }, .{ "date", "Mon, 21 Oct 2013 20:13:21 GMT" }, .{ "location", "https://www.example.com" }];
    third := Field.[
        .{ ":status", "200" }, .{ "cache-control", "private" }, .{ "date", "Mon, 21 Oct 2013 20:13:22 GMT" }, .{ "location", "https://www.example.com" },
        .{ "content-encoding", "gzip" }, .{ "set-cookie", "foo=ASDJKHQKBZXOQWEOPIUAXQWEOIU; max-age=3600; version=1" },
    ];

    // C.5: a table of 256 bytes, so entries are evicted; no Huffman coding.
    plain: Hpack_Decoder;

    hpack_decoder_init(*plain, 256);
    defer hpack_decoder_free(*plain);

    expect_block(*plain, "4803 3330 3258 0770 7269 7661 7465 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3120 474d 546e 1768 7474 7073 3a2f 2f77 7777 2e65 7861 6d70 6c65 2e63 6f6d", first, 222, "C.5.1");
    expect_block(*plain, "4803 3330 37c1 c0bf", second, 222, "C.5.2");
    expect_block(*plain, "88c1 611d 4d6f 6e2c 2032 3120 4f63 7420 3230 3133 2032 303a 3133 3a32 3220 474d 54c0 5a04 677a 6970 7738 666f 6f3d 4153 444a 4b48 514b 425a 584f 5157 454f 5049 5541 5851 5745 4f49 553b 206d 6178 2d61 6765 3d33 3630 303b 2076 6572 7369 6f6e 3d31", third, 215, "C.5.3");

    // C.6: the same with Huffman coding.
    huffman: Hpack_Decoder;

    hpack_decoder_init(*huffman, 256);
    defer hpack_decoder_free(*huffman);

    expect_block(*huffman, "4882 6402 5885 aec3 771a 4b61 96d0 7abe 9410 54d4 44a8 2005 9504 0b81 66e0 82a6 2d1b ff6e 919d 29ad 1718 63c7 8f0b 97c8 e9ae 82ae 43d3", first, 222, "C.6.1");
    expect_block(*huffman, "4883 640e ffc1 c0bf", second, 222, "C.6.2");
    expect_block(*huffman, "88c1 6196 d07a be94 1054 d444 a820 0595 040b 8166 e084 a62d 1bff c05a 839b d9ab 77ad 94e7 821d d7f2 e6c7 b335 dfdf cd5b 3960 d5af 2708 7f36 72c1 ab27 0fb5 291f 9587 3160 65c0 03ed 4ee5 b106 3d50 07", third, 215, "C.6.3");

    // The encoder makes the same blocks for the fields that may be indexed.
    encoder: Hpack_Encoder;

    hpack_encoder_init(*encoder, 256);
    defer hpack_encoder_free(*encoder);

    expect_encoding(*encoder, first, "4882 6402 5885 aec3 771a 4b61 96d0 7abe 9410 54d4 44a8 2005 9504 0b81 66e0 82a6 2d1b ff6e 919d 29ad 1718 63c7 8f0b 97c8 e9ae 82ae 43d3", "encoding C.6.1");
}

literals :: () {
    // C.2.1 to C.2.4: the four kinds of literal and an indexed field.
    decoder: Hpack_Decoder;

    hpack_decoder_init(*decoder);
    defer hpack_decoder_free(*decoder);

    expect_block(*decoder, "400a 6375 7374 6f6d 2d6b 6579 0d63 7573 746f 6d2d 6865 6164 6572", Field.[.{ "custom-key", "custom-header" }], 55, "C.2.1");
    expect_block(*decoder, "040c 2f73 616d 706c 652f 7061 7468", Field.[.{ ":path", "/sample/path" }], 55, "C.2.2");
    expect_block(*decoder, "1008 7061 7373 776f 7264 0673 6563 7265 74", Field.[.{ "password", "secret" }], 55, "C.2.3");
    expect_block(*decoder, "82", Field.[.{ ":method", "GET" }], 55, "C.2.4");
}

// Every byte value survives the Huffman code, alone and in a mixed string.
huffman_round_trip :: () {
    for 0..255 {
        text := string.{ 1, *cast(u8) it };
        out: [..] u8;
        back: [..] u8;

        hpack_huffman_encode(*out, text);

        check(hpack_huffman_decode(out, *back), tprint("byte % decodes", it));
        check(back.count == 1 && back[0] == cast(u8) it, tprint("byte % comes back", it));
        check(out.count == hpack_huffman_size(text), tprint("byte % has the size it reports", it));

        array_reset(*out);
        array_reset(*back);
    }

    everything: [256] u8;

    for 0..255 everything[it] = cast(u8) ((it * 7 + 3) & 0xff);

    text := string.{ 256, everything.data };
    out: [..] u8;
    back: [..] u8;

    defer array_reset(*out);
    defer array_reset(*back);

    hpack_huffman_encode(*out, text);

    check(hpack_huffman_decode(out, *back) && string.{ back.count, back.data } == text, "a string of all bytes comes back");

    // Padding must be ones and under eight bits; the end-of-string code is not a character.
    bad: [..] u8;

    defer array_reset(*bad);

    check(!hpack_huffman_decode(hex_bytes("f1e3 c2e5 f23a 6ba0 ab90 f400"), *bad), "padding of zeros is refused");
    check(!hpack_huffman_decode(hex_bytes("f1e3 c2e5 f23a 6ba0 ab90 f4ff ff"), *bad), "padding of a whole byte is refused");
    check(!hpack_huffman_decode(hex_bytes("ffff ffff"), *bad), "the end-of-string code is refused");
}

integers :: () {
    // C.1: 10 in five bits, 1337 in five bits, 42 at an octet boundary.
    out: [..] u8;

    defer array_reset(*out);

    hpack_put_integer(*out, 5, 0, 10);
    check(hex_of(out) == "0a", "10 with a 5-bit prefix");

    out.count = 0;
    hpack_put_integer(*out, 5, 0, 1337);
    check(hex_of(out) == "1f9a0a", "1337 with a 5-bit prefix");

    out.count = 0;
    hpack_put_integer(*out, 8, 0, 42);
    check(hex_of(out) == "2a", "42 with an 8-bit prefix");

    for value: s64.[0, 1, 30, 31, 32, 127, 128, 255, 256, 16383, 16384, 2097151, 2097152, 1 << 30, 0x7fffffff] {
        for prefix: 4..7 {
            out.count = 0;
            hpack_put_integer(*out, prefix, 0, value);

            at := 0;
            back, ok := hpack_read_integer(out, *at, prefix);

            check(ok && back == value && at == out.count, tprint("% survives a % bit prefix (got %)", value, prefix, back));
        }
    }

    // Too long and cut off.
    at := 0;
    _, ok := hpack_read_integer(hex_bytes("1f ff ff ff ff ff 01"), *at, 5);

    check(!ok, "an integer of more than 31 bits is refused");

    at = 0;
    _, ok = hpack_read_integer(hex_bytes("1f ff"), *at, 5);

    check(!ok, "an integer that stops in the middle is refused");
}

bad_blocks :: () {
    decoder: Hpack_Decoder;

    hpack_decoder_init(*decoder);
    defer hpack_decoder_free(*decoder);

    refuse :: (decoder: *Hpack_Decoder, hex: string, what: string) {
        block: Hpack_Block;
        defer hpack_block_free(*block);

        check(hpack_decode(decoder, hex_bytes(hex), 1 << 20, *block) == .COMPRESSION_ERROR, what);
    }

    refuse(*decoder, "80", "index 0");
    refuse(*decoder, "be", "an index beyond the table");
    refuse(*decoder, "ff ff ff ff ff 7f", "an index that does not fit");
    refuse(*decoder, "400a 6375 7374", "a string cut short");
    refuse(*decoder, "0f", "a name index cut short");
    refuse(*decoder, "8240 0161 0161 3fe1 1f", "a table size change after a field");
    refuse(*decoder, "3fe2 1f", "a table size above the limit");

    // A size change at the start is fine, and so is one that empties the table.
    block: Hpack_Block;

    defer hpack_block_free(*block);

    check(hpack_decode(*decoder, hex_bytes("20 82"), 1 << 20, *block) == .NO_ERROR && block.fields.count == 1, "a table size of 0 at the start");

    // A block over the size limit reports it and still keeps the table in step.
    hpack_block_free(*block);

    check(hpack_decode(*decoder, hex_bytes("8286 8441 8cf1 e3c2 e5f2 3a6b a0ab 90f4 ff"), 100, *block) == .NO_ERROR && block.too_large, "a block over the limit is flagged");
}

// A table that cannot hold an entry, and one that is resized while full.
tables :: () {
    encoder: Hpack_Encoder;

    hpack_encoder_init(*encoder, 100);
    defer hpack_encoder_free(*encoder);

    out: [..] u8;

    defer array_reset(*out);

    // 40 + 32 fits; 90 + 32 does not and is sent without indexing.
    hpack_encode_field(*encoder, *out, "a", "0123456789012345678901234567890123456789");
    check(encoder.table.count == 1, "a small field is indexed");

    hpack_encode_field(*encoder, *out, "b", "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");
    check(encoder.table.count == 1 && encoder.table.size == 73, "a field too large for the table is left out of it");

    // The peer shrinks the table: the next block says so first.
    hpack_encoder_limit(*encoder, 0);
    out.count = 0;
    hpack_encode_start(*encoder, *out);

    check(hex_of(out) == "20" && encoder.table.count == 0, "a smaller table is announced and applied");

    // Credentials stay out of the table.
    hpack_encoder_limit(*encoder, 4096);
    out.count = 0;
    hpack_encode_start(*encoder, *out);
    hpack_encode_field(*encoder, *out, "authorization", "Bearer abc");

    check(encoder.table.count == 0 && (out[out.count - 11] & 0xf0) != 0x40, "authorization is never indexed");

    // Many entries cycle through the ring.
    decoder: Hpack_Decoder;

    hpack_decoder_init(*decoder);
    defer hpack_decoder_free(*decoder);

    sender: Hpack_Encoder;

    hpack_encoder_init(*sender);
    defer hpack_encoder_free(*sender);

    for 1..500 {
        out.count = 0;

        name := tprint("x-name-%", it % 37);
        value := tprint("value-%", it);

        hpack_encode_start(*sender, *out);
        hpack_encode_field(*sender, *out, name, value);
        hpack_encode_field(*sender, *out, name, value);

        block: Hpack_Block;

        error := hpack_decode(*decoder, out, 1 << 20, *block);

        check(error == .NO_ERROR && block.fields.count == 2 && block.fields[0].name == name && block.fields[1].value == value, tprint("round trip %", it));

        hpack_block_free(*block);
    }

    check(decoder.table.size == sender.table.size && decoder.table.count == sender.table.count, "both tables end in step");
}

main :: () {
    requests();
    responses();
    literals();
    huffman_round_trip();
    integers();
    bad_blocks();
    tables();

    finish();
}
