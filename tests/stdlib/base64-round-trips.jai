// Base64 at run time: the RFC 4648 vectors with and without padding, base64url, custom
// alphabets, strict decoding (bad lengths, stray bits, characters after the padding) and round
// trips of every byte value over many lengths.
#import "Basic";
#import "Base64";

main :: () {
    rfc_inputs := string.["", "f", "fo", "foo", "foob", "fooba", "foobar"];
    rfc_padded := string.["", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"];
    rfc_bare := string.["", "Zg", "Zm8", "Zm9v", "Zm9vYg", "Zm9vYmE", "Zm9vYmFy"];
    for rfc_inputs {
        padded := base64_encode(it);
        assert(padded == rfc_padded[it_index], "encode '%': '%'", it, padded);
        bare := base64_encode(it, pad = false);
        assert(bare == rfc_bare[it_index], "encode without padding '%': '%'", it, bare);
        decoded, ok := base64_decode(padded);
        assert(ok && decoded == it, "decode '%': '%' %", padded, decoded, ok);
        free(decoded);
        decoded, ok = base64_decode(bare);
        assert(ok && decoded == it, "decode unpadded '%'", bare);
        free(decoded);
        free(padded);
        free(bare);
    }

    // The two alphabets differ only in the last two symbols.
    tricky := "\xfb\xff\xfe";
    standard := base64_encode(tricky);
    url := base64url_encode(tricky);
    assert(standard == "+//+" && url == "-__-", "% %", standard, url);
    decoded, ok := base64url_decode(url);
    assert(ok && decoded == tricky);
    free(decoded);
    _, ok = base64url_decode(standard);
    assert(!ok, "'+' and '/' are not base64url");
    _, ok = base64_decode(url);
    assert(!ok, "'-' and '_' are not standard base64");
    url_padded := base64_encode("\xff", BASE64URL_ENCODER_TABLE);
    assert(url_padded == "_w==", "the table argument with padding: %", url_padded);
    assert(base64url_encode("\xff") == "_w", "base64url_encode never pads");
    free(standard);
    free(url);
    free(url_padded);

    // Whitespace is skipped; everything else that is not in the alphabet fails.
    decoded, ok = base64_decode("Zm9v\r\nYm Fy\t");
    assert(ok && decoded == "foobar");
    free(decoded);
    bad := string.["Z", "Zg=", "Zg===", "Zm9vY", "Zh==", "Zm9=", "Zm9v!", "Zg==Zg==", "Z=g=", "====", "Zm9vYmF=y"];
    for bad {
        result, success := base64_decode(it);
        assert(!success && result == "", "'%' must be rejected (got '%')", it, result);
    }

    // Custom alphabets.
    reversed := "/+9876543210zyxwvutsrqponmlkjihgfedcbaZYXWVUTSRQPONMLKJIHGFEDCBA";
    custom := base64_encode_with_alphabet("foobar", reversed, true);
    assert(custom.count == 8);
    back, back_ok := base64_decode_with_alphabet(custom, reversed);
    assert(back_ok && back == "foobar", "custom alphabet round trip: '%' -> '%'", custom, back);
    free(custom);
    free(back);
    assert(base64_encode_with_alphabet("x", "too short", true) == "", "an alphabet needs 64 symbols");
    duplicate := "AACDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    assert(base64_encode_with_alphabet("x", duplicate, true) == "", "an alphabet needs distinct symbols");
    _, back_ok = base64_decode_with_alphabet("QQ==", duplicate);
    assert(!back_ok);
    _, back_ok = base64_decode_with_alphabet("QQ==", "short");
    assert(!back_ok);
    index, found := base64_index_of(#char "/", BASE64_ALPHABET);
    assert(found && index == 63);
    index, found = base64_index_of(#char "-", BASE64_ALPHABET);
    assert(!found && index == -1);
    index, found = base64_index_of(#char "-", BASE64URL_ALPHABET);
    assert(found && index == 62);

    // Round trips of every byte value at every length up to 300, both alphabets.
    bytes: [300] u8;
    for *bytes it.* = cast,trunc(u8) (it_index * 37 + 11);
    for length: 0..bytes.count {
        input := string.{ length, bytes.data };
        encoded := base64_encode(input);
        assert(encoded.count == (length + 2) / 3 * 4, "padded length for %", length);
        output, success := base64_decode(encoded);
        assert(success && output == input, "round trip of % bytes", length);
        free(encoded);
        free(output);
        encoded = base64url_encode(input);
        assert(encoded.count == (length * 4 + 2) / 3, "unpadded length for %", length);
        output, success = base64url_decode(encoded);
        assert(success && output == input, "base64url round trip of % bytes", length);
        free(encoded);
        free(output);
    }

    test_base64_module();
    print("ok\n");
}
