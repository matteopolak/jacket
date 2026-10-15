// Reviewed lowercase encoding, numeric array order and independent allocations.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    data: [5] u8 = .[0, 1, 15, 128, 255];
    text := A.to_hex_string(string.{ data.count, data.data });

    defer free(text);
    assert(text == "00010f80ff");
    data[0] = 255;
    assert(text == "00010f80ff");
    other := A.to_hex_string(string.{ data.count, data.data });

    defer free(other);
    assert(other == "ff010f80ff" && other.data != text.data);
    text.data[0] = #char "9";
    assert(other == "ff010f80ff");

    empty := A.to_hex_string("");

    defer free(empty);
    assert(empty.count == 0);

    bytes: [8] u8 = .[0, 1, 15, 16, 127, 128, 254, 255];
    byte_text := A.to_hex_string(bytes);

    defer free(byte_text);
    assert(byte_text == "00010f107f80feff");

    words: [8] u16 = .[0, 1, 15, 256, 0x1234, 0x8000, 0xfffe, 0xffff];
    word_text := A.to_hex_string(words);

    defer free(word_text);
    assert(word_text == "00000001000f010012348000fffeffff");

    wide: [8] u32 = .[0, 1, 15, 256, 0x12345678, 0x80000000, 0xfffffffe, 0xffffffff];
    wide_text := A.to_hex_string(wide);

    defer free(wide_text);
    assert(wide_text == "00000000000000010000000f000001001234567880000000fffffffeffffffff");

    largest: [8] u64 = .[0, 1, 15, 256, 0x123456789abcdef0, 0x8000000000000000, 0xfffffffffffffffe, 0xffffffffffffffff];
    largest_text := A.to_hex_string(largest);

    defer free(largest_text);
    assert(largest_text == "00000000000000000000000000000001000000000000000f0000000000000100123456789abcdef08000000000000000fffffffffffffffeffffffffffffffff");
    signed: [8] s8 = .[-1, -128, -2, 0, 1, 2, 3, 127];
    signed_text := A.to_hex_string(signed);

    defer free(signed_text);
    assert(signed_text == "ff80fe000102037f");
    print("ok\n");
}
