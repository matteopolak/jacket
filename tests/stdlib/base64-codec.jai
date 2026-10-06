// Base64: the RFC 4648 vectors in both alphabets, padding on and off, binary round trips of
// every length, custom alphabets, whitespace, and the inputs a decoder must reject.
#import "Basic";
#import "Base64";

main :: () {
    vectors :: string.["", "f", "fo", "foo", "foob", "fooba", "foobar"];
    padded :: string.["", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"];
    unpadded :: string.["", "Zg", "Zm8", "Zm9v", "Zm9vYg", "Zm9vYmE", "Zm9vYmFy"];
    for vectors {
        encoded := base64_encode(it);
        assert(encoded == padded[it_index], "encode '%' gave '%'", it, encoded);
        bare := base64_encode(it, pad = false);
        assert(bare == unpadded[it_index], "unpadded '%' gave '%'", it, bare);
        decoded, ok := base64_decode(encoded);
        assert(ok && decoded == it, "decode '%'", encoded);
        decoded, ok = base64_decode(bare);
        assert(ok && decoded == it, "decode unpadded '%'", bare);
    }

    // Bytes that differ between the alphabets: 0xFB 0xFF 0xBF encode to "+/+/" and "-_-_".
    special := "\xfb\xff\xbf";
    assert(base64_encode(special) == "+/+/");
    url := base64url_encode(special);
    assert(url == "-_-_", "base64url '%'", url);
    decoded, ok := base64url_decode(url);
    assert(ok && decoded == special);
    // Each decoder rejects the other's alphabet.
    _, ok = base64_decode("-_-_");
    assert(!ok);
    _, ok = base64url_decode("+/+/");
    assert(!ok);
    // base64url leaves padding off, and accepts it when present.
    assert(base64url_encode("f") == "Zg");
    decoded, ok = base64url_decode("Zg==");
    assert(ok && decoded == "f");

    // Every byte value, at every length up to 300, round trips.
    data: [300] u8;
    for i: 0..data.count - 1 data[i] = cast,trunc(u8) (i * 37 + 11);
    for length: 0..data.count {
        input := string.{ length, data.data };
        encoded := base64_encode(input);
        assert(encoded.count == (length + 2) / 3 * 4, "length % encodes to %", length, encoded.count);
        back, back_ok := base64_decode(encoded);
        assert(back_ok && back == input, "round trip of %", length);
        free(encoded);
        free(back);
    }

    // Whitespace (as in wrapped MIME text) is skipped.
    decoded, ok = base64_decode("Zm9v\r\nYmFy\n");
    assert(ok && decoded == "foobar");
    decoded, ok = base64_decode(" Zm 9v\tYg == ");
    assert(ok && decoded == "foob", "spaced '%'", decoded);

    // Malformed input fails without a partial result.
    bad :: string.["Z", "Zg=", "Zg===", "Zm9=v", "Zm9vY", "Zm!v", "Zh==", "Zm9w=", "=Zm9", "Zg==Zg=="];
    for bad {
        out, out_ok := base64_decode(it);
        assert(!out_ok && out.count == 0, "'%' is rejected", it);
    }

    // A custom alphabet.
    REVERSED :: "/+9876543210zyxwvutsrqponmlkjihgfedcbaZYXWVUTSRQPONMLKJIHGFEDCBA";
    custom := base64_encode_with_alphabet("foobar!", REVERSED, true);
    assert(custom.count == 12 && custom != base64_encode("foobar!"), "custom '%'", custom);
    decoded, ok = base64_decode_with_alphabet(custom, REVERSED);
    assert(ok && decoded == "foobar!", "custom round trip '%'", decoded);
    assert(base64_encode_with_alphabet("x", REVERSED, false).count == 2);
    // Alphabets must have 64 distinct symbols.
    assert(base64_encode_with_alphabet("x", "short", true) == "");
    duplicated := "AACDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    assert(base64_encode_with_alphabet("x", duplicated, true) == "");
    _, ok = base64_decode_with_alphabet("QQ==", duplicated);
    assert(!ok);
    _, ok = base64_decode_with_alphabet("QQ==", "short");
    assert(!ok);
    index, found := base64_index_of(#char "/", BASE64_ALPHABET);
    assert(found && index == 63);
    index, found = base64_index_of(#char "-", BASE64_ALPHABET);
    assert(!found && index == -1);

    test_base64_module();
    print("ok\n");
}
