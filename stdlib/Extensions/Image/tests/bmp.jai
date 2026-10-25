// BMP: hand-built files for each header, depth, bit field and RLE form, then round trips.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

// File header, a 40-byte info header and `extra` bytes (masks, palette) before the pixels.
start :: (b: *Builder, width: s64, height: s64, bits: s64, compression: s64, colors: s64, extra: [] u8, pixel_bytes: s64) {
    put_all(b, #char "B", #char "M");
    put32(b, 14 + 40 + extra.count + pixel_bytes);
    put32(b, 0);
    put32(b, 14 + 40 + extra.count);
    put32(b, 40);
    put32(b, width);
    put32(b, height);
    put16(b, 1);
    put16(b, bits);
    put32(b, compression);
    put32(b, pixel_bytes);
    put32(b, 2835);
    put32(b, 2835);
    put32(b, colors);
    put32(b, 0);
    for extra put8(b, it);
}

decode :: (b: Builder, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> Image, bool {
    image, ok := decode_bmp(b.bytes, max_pixels);
    return image, ok;
}

// Four BGRX entries: blue, green, red and (30, 20, 10).
PALETTE :: u8.[255, 0, 0, 0, 0, 255, 0, 0, 0, 0, 255, 0, 10, 20, 30, 0];

main :: () {
    // 24-bit, bottom-up, 2x2: rows padded to 8 bytes.
    b: Builder;

    start(*b, 2, 2, 24, 0, 0, .[], 16);
    put_all(*b, 1, 2, 3, 4, 5, 6, 0, 0); // bottom row: BGR pairs
    put_all(*b, 7, 8, 9, 10, 11, 12, 0, 0); // top row
    image, ok := decode(b);

    check(ok && image.format == .RGB8, "24-bit decodes");
    check(same_bytes(image.pixels, u8.[9, 8, 7, 12, 11, 10, 3, 2, 1, 6, 5, 4]), "24-bit bottom-up");
    deinit(*image);

    // The same with a negative height: rows are top-down.
    b2: Builder;

    start(*b2, 2, -2, 24, 0, 0, .[], 16);
    for b.bytes if it_index >= 54 put8(*b2, it);

    image, ok = decode(b2);
    check(ok && same_bytes(image.pixels, u8.[3, 2, 1, 6, 5, 4, 9, 8, 7, 12, 11, 10]), "24-bit top-down");
    deinit(*image);

    // 8-bit palette, 3x2, row stride 4.
    b3: Builder;

    start(*b3, 3, 2, 8, 0, 4, PALETTE, 8);
    put_all(*b3, 0, 1, 2, 0, 3, 3, 0, 0);
    image, ok = decode(b3);
    check(ok && image.format == .RGB8, "8-bit decodes");
    check(same_bytes(image.pixels, u8.[30, 20, 10, 30, 20, 10, 0, 0, 255, 0, 0, 255, 0, 255, 0, 255, 0, 0]), "8-bit palette");
    deinit(*image);

    // 4-bit: two pixels per byte, high nibble first; 3 pixels in a 4-byte row.
    b4: Builder;

    start(*b4, 3, 1, 4, 0, 4, PALETTE, 4);
    put_all(*b4, 0x01, 0x20, 0, 0);
    image, ok = decode(b4);
    check(ok && same_bytes(image.pixels, u8.[0, 0, 255, 0, 255, 0, 255, 0, 0]), "4-bit palette");
    deinit(*image);

    // 1-bit, 10 pixels: 1010101010.
    b5: Builder;

    start(*b5, 10, 1, 1, 0, 2, u8.[0, 0, 0, 0, 255, 255, 255, 0], 4);
    put_all(*b5, 0xaa, 0x80, 0, 0);
    image, ok = decode(b5);
    check(ok && image.pixels.count == 30, "1-bit decodes");
    check(image.pixels[0] == 255 && image.pixels[3] == 0 && image.pixels[27] == 0 && image.pixels[24] == 255, "1-bit pixels");
    deinit(*image);

    // 16-bit default 5-5-5: 0x7c00 is red, 0x03e0 green, 0x001f blue.
    b6: Builder;

    start(*b6, 3, 1, 16, 0, 0, .[], 8);
    put_all(*b6, 0x00, 0x7c, 0xe0, 0x03, 0x1f, 0x00, 0, 0);
    image, ok = decode(b6);
    check(ok && same_bytes(image.pixels, u8.[255, 0, 0, 0, 255, 0, 0, 0, 255]), "16-bit 5-5-5");
    deinit(*image);

    // 16-bit 5-6-5 through BI_BITFIELDS.
    masks565: Builder;

    put32(*masks565, 0xf800);
    put32(*masks565, 0x07e0);
    put32(*masks565, 0x001f);
    b7: Builder;

    start(*b7, 2, 1, 16, 3, 0, masks565.bytes, 4);
    put_all(*b7, 0xe0, 0x07, 0xff, 0xff);
    image, ok = decode(b7);
    check(ok && same_bytes(image.pixels, u8.[0, 255, 0, 255, 255, 255]), "16-bit 5-6-5");
    deinit(*image);

    // 32-bit without masks: an all-zero fourth byte is padding, anything else is alpha.
    b8: Builder;

    start(*b8, 2, 1, 32, 0, 0, .[], 8);
    put_all(*b8, 3, 2, 1, 0, 6, 5, 4, 0);
    image, ok = decode(b8);
    check(ok && image.format == .RGB8 && same_bytes(image.pixels, u8.[1, 2, 3, 4, 5, 6]), "32-bit without alpha");
    deinit(*image);

    b9: Builder;

    start(*b9, 2, 1, 32, 0, 0, .[], 8);
    put_all(*b9, 3, 2, 1, 200, 6, 5, 4, 0);
    image, ok = decode(b9);
    check(ok && image.format == .RGBA8 && same_bytes(image.pixels, u8.[1, 2, 3, 200, 4, 5, 6, 0]), "32-bit with alpha");
    deinit(*image);

    // 32-bit BI_BITFIELDS with an alpha mask in the top byte (masks follow the info header).
    mask_rgba: Builder;

    put32(*mask_rgba, 0x00ff0000);
    put32(*mask_rgba, 0x0000ff00);
    put32(*mask_rgba, 0x000000ff);
    put32(*mask_rgba, 0xff000000);
    // BITMAPINFOHEADER cannot hold four masks, so use compression 6 (BI_ALPHABITFIELDS).
    b10: Builder;

    start(*b10, 1, 1, 32, 6, 0, mask_rgba.bytes, 4);
    put_all(*b10, 3, 2, 1, 77);
    image, ok = decode(b10);
    check(ok && image.format == .RGBA8 && same_bytes(image.pixels, u8.[1, 2, 3, 77]), "32-bit alpha bit fields");
    deinit(*image);

    // RLE8: run of 3 x index 1, escape (end of line), literal run of 3 (padded), end of bitmap.
    b11: Builder;

    start(*b11, 3, 2, 8, 1, 4, PALETTE, 12);
    put_all(*b11, 3, 1, 0, 0, 0, 3, 2, 0, 3, 0, 0, 1); // the bottom row first
    image, ok = decode(b11);
    check(ok && same_bytes(image.pixels, u8.[255, 0, 0, 0, 0, 255, 30, 20, 10, 0, 255, 0, 0, 255, 0, 0, 255, 0]), "RLE8");
    deinit(*image);

    // RLE4: a run of 4 alternating indices 1 and 2 ("0x12" repeated), then end of bitmap.
    b12: Builder;

    start(*b12, 4, 1, 4, 2, 4, PALETTE, 4);
    put_all(*b12, 4, 0x12, 0, 1);
    image, ok = decode(b12);
    check(ok && same_bytes(image.pixels, u8.[0, 255, 0, 255, 0, 0, 0, 255, 0, 255, 0, 0]), "RLE4");
    deinit(*image);

    // OS/2 core header: 12 bytes, 16-bit sizes, 3-byte palette entries.
    b13: Builder;

    put_all(*b13, #char "B", #char "M");
    put32(*b13, 14 + 12 + 6 + 4);
    put32(*b13, 0);
    put32(*b13, 14 + 12 + 6);
    put32(*b13, 12);
    put16(*b13, 2);
    put16(*b13, 1);
    put16(*b13, 1);
    put16(*b13, 8);
    put_all(*b13, 1, 2, 3, 4, 5, 6); // two palette entries (BGR)
    put_all(*b13, 1, 0, 0, 0);
    image, ok = decode(b13);
    check(ok && same_bytes(image.pixels, u8.[6, 5, 4, 3, 2, 1]), "OS/2 header");
    deinit(*image);

    // Malformed.
    for 0..b.bytes.count - 1 {
        cut: Builder;
        for 0..it - 1 put8(*cut, b.bytes[it]);
        _, cut_ok := decode(cut);
        check(!cut_ok, "every truncation of a 24-bit file is refused");
    }

    huge: Builder;

    start(*huge, 100000, 100000, 24, 0, 0, .[], 0);
    _, huge_ok := decode(huge);

    check(!huge_ok, "huge size refused");

    _, limited := decode(b, 3);

    check(!limited, "max_pixels");
    _, limited = decode(b, 4);
    check(limited, "max_pixels exactly");

    // Round trips.
    for format: ALL_FORMATS for size : s32.[1, 2, 3, 5, 17] {
        source := random_image(size, size + 1, format, cast(u64) size);
        data := encode_bmp(source);
        decoded, decoded_ok, error := decode_bmp(data);

        check(decoded_ok, error);

        wanted := ifx has_alpha(format) then Pixel_Format.RGBA8 else .RGB8;

        check(decoded.format == wanted, "bmp format");
        reference := convert(source, wanted);

        check(same_bytes(decoded.pixels, reference.pixels), "bmp round trip");

        array_free(data);
        deinit(*source);
        deinit(*decoded);
        deinit(*reference);
    }

    finish();
}
