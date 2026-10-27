// UTF-8 encoding and decoding with the Unicode module over mixed one- to four-byte text. The text
// is made once, on the first call.
#import "Basic";
#import "Unicode";
#import "Extensions/Testing";

UTF8_CHARACTERS :: 1_500_000;

utf8_text: string;

// One to four bytes per character: about half ASCII, the rest two-, three- and four-byte.
make_utf8_text :: () {
    builder: String_Builder;
    state: u32 = 5;
    scratch: [4] u8;

    for 0..UTF8_CHARACTERS - 1 {
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

    utf8_text = builder_to_string(*builder);
}

utf8_encode :: (b: *Bench) {
    set_items(b, UTF8_CHARACTERS);

    for 0..b.iterations - 1 {
        builder: String_Builder;
        state: u32 = 5;
        scratch: [4] u8;

        for 0..UTF8_CHARACTERS - 1 {
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

        black_box(builder_string_length(*builder));
        free_buffers(*builder);
    }
} @bench @samples(20)

utf8_decode :: (b: *Bench) {
    if !utf8_text.count make_utf8_text();

    reset_timer(b);
    set_items(b, UTF8_CHARACTERS);
    set_bytes(b, utf8_text.count);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;
        rest := utf8_text;

        while rest.count {
            code, length, result := character_utf8_to_utf32(rest.data, rest.count);
            if result != .CONVERSION_OK break;

            checksum = (checksum ^ code) * 1099511628211;
            rest.data += length;
            rest.count -= length;
        }

        black_box(checksum);
    }
} @bench @samples(20)

utf8_next :: (b: *Bench) {
    if !utf8_text.count make_utf8_text();

    reset_timer(b);
    set_items(b, UTF8_CHARACTERS);
    set_bytes(b, utf8_text.count);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;
        rest := utf8_text;

        while rest.count {
            code, result := utf8_next_character(*rest);
            if result != .CONVERSION_OK break;
            checksum = (checksum ^ code) * 1099511628211;
        }

        black_box(checksum);
    }
} @bench @samples(20)
