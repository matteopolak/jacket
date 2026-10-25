// PNM: P2, P3, P5 and P6 with comments, depths above 8 bits, malformed headers; encode_ppm.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

decode :: (text: string, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> Image, bool {
    image, ok := decode_pnm(bytes_of(text), max_pixels);
    return image, ok;
}

main :: () {
    // P2 with comments between and inside tokens' gaps; maxval 15 scales to 255.
    image, ok := decode("P2\n# a comment\n3 2 # another\n15\n0 5 10\n15 1 14\n");

    check(ok && image.format == .GRAY8 && image.width == 3 && image.height == 2, "P2 header");
    check(same_bytes(image.pixels, u8.[0, 85, 170, 255, 17, 238]), "P2 samples scaled");
    deinit(*image);

    // P3.
    image, ok = decode("P3 2 1 255 1 2 3\n  4 5 6");
    check(ok && image.format == .RGB8 && same_bytes(image.pixels, u8.[1, 2, 3, 4, 5, 6]), "P3");
    deinit(*image);

    // P5: one whitespace byte after maxval, then raw samples (which may look like whitespace).
    image, ok = decode("P5\n2 2\n255\n\x00\x0a\x20\xff");
    check(ok && same_bytes(image.pixels, u8.[0, 10, 32, 255]), "P5");
    deinit(*image);

    // P6 with 16-bit big-endian samples: maxval 65535 keeps the high byte, rounded.
    image, ok = decode("P6\n1 1\n65535\n\xff\xff\x80\x00\x00\x00");
    check(ok && same_bytes(image.pixels, u8.[255, 128, 0]), "P6 16-bit");
    deinit(*image);

    // maxval 1.
    image, ok = decode("P5 4 1 1\n\x00\x01\x01\x00");
    check(ok && same_bytes(image.pixels, u8.[0, 255, 255, 0]), "P5 maxval 1");
    deinit(*image);

    // Refusals.
    bad := string.[
        "", "P", "P7\n1 1\n255\n\x00", "P5\n1 1\n255\n", "P5\n1 1\n0\n\x00", "P5\n1 1\n65536\n\x00\x00",
        "P2\n1 1\n255\n256", "P2\n1 1\n255\n", "P3\n1 1\n255\n1 2", "P5\n-1 1\n255\n\x00", "P5\n0 1\n255\n",
        "P5\n99999999999 1\n255\n\x00", "P5\n2147483647 2147483647\n255\n\x00", "P6\n1 1\n255\x00\x01\x02\x03", "P5\n1 1\n300\n\x00",
        "P5\n1 1\n65535\n\x00",
    ];
    for bad {
        _, accepted := decode(it);
        check(!accepted, tprint("rejects %", it_index));
    }

    _, accepted := decode("P5\n4 4\n255\n0123456789abcdef", 15);

    check(!accepted, "max_pixels");
    _, accepted = decode("P5\n4 4\n255\n0123456789abcdef", 16);
    check(accepted, "max_pixels exactly");

    // Encoding.
    for format: ALL_FORMATS {
        source := random_image(5, 3, format, 3);
        data := encode_ppm(source);
        decoded, decoded_ok, error := decode_pnm(data);

        check(decoded_ok, error);

        gray := format == .GRAY8 || format == .GRAY_ALPHA8;

        check(decoded.format == ifx gray then Pixel_Format.GRAY8 else .RGB8, "ppm format");
        reference := convert(convert(source, ifx gray then Pixel_Format.GRAY8 else .RGB8), decoded.format);

        check(same_bytes(decoded.pixels, reference.pixels), "ppm round trip");
        check(data[1] == ifx gray then #char "5" else #char "6", "magic");

        array_free(data);
        deinit(*source);
        deinit(*decoded);
        deinit(*reference);
    }

    finish();
}
