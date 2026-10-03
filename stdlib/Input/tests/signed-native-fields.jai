#import "Basic";
#load "../../internal/input/native-fields.jai";

main :: () {
    // Exhaust every signed word, including the issue #1 encoding 65416 (-120).
    for encoded: 0..65535 {
        expected := encoded;
        if encoded >= 32768 expected -= 65536;

        assert(signed_word(cast(u64) encoded) == expected);
        narrow: s32 = cast(s32) encoded;
        native_word: u64 = cast(u64) narrow;
        packed := 0xfedcba980000007f | (native_word << 16);

        assert((packed >> 32) == 0xfedcba98);

        assert(high_signed_word(packed) == expected);
        assert(signed_word(packed) == 127);
    }

    assert(high_signed_word(0x12345678ff88000c) == -120);
    assert(signed_word(0x80008000) == -32768);
    assert(high_signed_word(0x80008000) == -32768);
    print("ok\n");
}
