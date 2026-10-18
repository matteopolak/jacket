// A scalar value encodes to UTF-8 that decodes back to itself, consuming exactly what was written.
fuzz_unicode_round_trip :: (r: *Rng) {
    code := gen(r, u32) % 0x110000;
    if code >= 0xD800 && code < 0xE000 return;

    encoded := Target.character_utf32_to_utf8(code);
    cursor := encoded;
    decoded, status := Target.utf8_next_character(*cursor);
    assert(status == .CONVERSION_OK && decoded == code, "utf8 round trip value");
    assert(cursor.count == 0, "utf8 round trip consumes the whole encoding");
}

// Decoding arbitrary bytes never reads past the buffer and always advances.
fuzz_unicode_decode_bounds :: (r: *Rng) {
    input := gen(r, string);
    cursor := input;
    steps := 0;
    while cursor.count > 0 {
        before := cursor.count;
        Target.utf8_next_character(*cursor);
        assert(cursor.count < before && cursor.count >= 0, "decoder advances within the buffer");
        steps += 1;
    }
    assert(steps <= input.count);
}

// The pointer-and-length decoder reports a length within the buffer, whatever the bytes are.
fuzz_unicode_decode_length :: (r: *Rng) {
    input := gen(r, string);
    if input.count == 0 return;
    code, length, status := Target.character_utf8_to_utf32(input.data, input.count);
    assert(length >= 0 && length <= input.count, "decoded length stays within the buffer");
}
