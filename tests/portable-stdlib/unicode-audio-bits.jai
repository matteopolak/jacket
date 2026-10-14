#import "Basic";
U :: #import "Unicode";
A :: #import "Adpcm";
B :: #import "Bit_Array";

only_set :: B.only_set;
#assert is_constant(U.offsetsFromUTF8);
#assert is_constant(A.step_table);

main :: () {
    codes: [4] u32 = .[0x41, 0xa2, 0x20ac, 0x1f642];
    for code: codes {
        encoded := U.character_utf32_to_utf8(code);
        decoded, count, status := U.character_utf8_to_utf32(encoded.data, encoded.count, true);
        assert(status == .CONVERSION_OK && decoded == code && count == encoded.count);
        free(encoded);
    }

    bad := "\xc0\x80";
    code, count, status := U.character_utf8_to_utf32(bad.data, bad.count, true);

    assert(status == .SOURCE_ILLEGAL && code == 0xfffd && count == 2);
    bad = "\xe2x\x80";
    code, count, status = U.character_utf8_to_utf32(bad.data, bad.count, true);
    assert(status == .SOURCE_ILLEGAL && count == 0);
    block: [16] u8 = .[0xe8, 3, 0, 0, 0x18, 0xfc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    dest: [36] u8;
    written := A.IMA_ADPCM_decode(A.Adpcm_Info.{ 2, 16, 9 }, block, dest);

    assert(written == 36);

    for frame: 0..8 {
        assert(dest[frame * 4] == 0xe8 && dest[frame * 4 + 1] == 3);
        assert(dest[frame * 4 + 2] == 0x18 && dest[frame * 4 + 3] == 0xfc);
    }

    bits := B.make_bit_array(65);

    defer B.deinit(*bits);
    B.set_bit(*bits, 0);
    B.set_bit(*bits, 64);
    count_set: int;

    for: only_set bits {
        assert(it && (it_index == 0 || it_index == 64));
        count_set += 1;
    }

    assert(count_set == 2);
    bits[64] = false;
    assert(bits[0] && !bits[64]);
    print("ok\n");
}
