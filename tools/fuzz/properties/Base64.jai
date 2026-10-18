// Encoding then decoding returns the input, for both alphabets and with and without padding.
fuzz_base64_round_trip :: (r: *Rng) {
    input := gen(r, string);

    encoded := Target.base64_encode(input);
    decoded, ok := Target.base64_decode(encoded);
    assert(ok && decoded == input, "base64 round trip");

    url := Target.base64url_encode(input);
    url_decoded, url_ok := Target.base64url_decode(url);
    assert(url_ok && url_decoded == input, "base64url round trip");

    unpadded := Target.base64_encode(input, pad = false);
    unpadded_decoded, unpadded_ok := Target.base64_decode(unpadded);
    assert(unpadded_ok && unpadded_decoded == input, "unpadded base64 round trip");
}
