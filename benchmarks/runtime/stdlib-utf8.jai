// UTF-8 decoding and encoding with the Unicode module over mixed one- to four-byte text.
#import "Basic";
#import "Unicode";

main :: () {
    builder: String_Builder;
    state: u32 = 5;
    scratch: [4] u8;

    for 0..1_500_000 - 1 {
        state = state * 1664525 + 1013904223;
        pick := (state >> 24) % 8;
        code: u32;

        if pick < 4 code = 32 + (state >> 8) % 95;
        else if pick < 6 code = 0x100 + (state >> 8) % 0x600;
        else if pick < 7 code = 0x4E00 + (state >> 8) % 0x5000;
        else code = 0x1F300 + (state >> 8) % 0x300;

        text := string.{ 4, scratch.data };

        character_utf32_to_utf8(code, *text);
        append(*builder, text);
    }

    text := builder_to_string(*builder);
    checksum: u64 = 1469598103934665603;
    characters := 0;

    for round: 0..5 {
        rest := text;
        while rest.count {
            code, length, result := character_utf8_to_utf32(rest.data, rest.count);
            if result != .CONVERSION_OK break;

            checksum = (checksum ^ code) * 1099511628211;
            characters += 1;
            rest.data += length;
            rest.count -= length;
        }

        rest = text;
        while rest.count {
            code, result := utf8_next_character(*rest);
            if result != .CONVERSION_OK break;
            checksum = (checksum ^ code) * 1099511628211;
        }
    }

    print("% % %\n", text.count, characters, checksum);
}
