// QOI: one stream that uses every op, the end marker, and round trips of every pixel format.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

main :: () {
    // Eight RGBA pixels: RGB, DIFF, LUMA, RGBA, INDEX, RUN of three.
    b: Builder;

    put_all(*b, 0x71, 0x6f, 0x69, 0x66, 0, 0, 0, 8, 0, 0, 0, 1, 4, 0);
    put_all(*b, 0xfe, 10, 20, 30);
    put_all(*b, 0x76);
    put_all(*b, 0xaa, 0x5a);
    put_all(*b, 0xff, 1, 2, 3, 4);
    put_all(*b, 9);
    put_all(*b, 0xc2);
    put_all(*b, 0, 0, 0, 0, 0, 0, 0, 1);

    image, ok, error := decode_qoi(b.bytes);

    check(ok, error);
    check(image.width == 8 && image.height == 1 && image.format == .RGBA8, "header");
    expected := u8.[
        10, 20, 30, 255, 11, 19, 30, 255, 18, 29, 42, 255, 1, 2, 3, 4,
        10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255
    ];

    check(same_bytes(image.pixels, expected), "every op decodes");
    deinit(*image);

    // Round trips.
    sizes := s32.[1, 2, 7, 33, 64];
    for format: ALL_FORMATS for w : sizes for h : sizes {
        source := random_image(w, h, format, cast(u64) (w * 31 + h));
        data := encode_qoi(source);
        decoded, decoded_ok, decoded_error := decode_qoi(data);

        check(decoded_ok, decoded_error);

        wanted := ifx has_alpha(format) then Pixel_Format.RGBA8 else .RGB8;

        check(decoded.format == wanted, "decoded format");
        reference := convert(source, wanted);

        check(same_bytes(decoded.pixels, reference.pixels), "qoi round trip");

        via_detect, detect_ok, _ := decode_image(data);

        check(detect_ok && same_bytes(via_detect.pixels, reference.pixels), "decode_image finds qoi");

        array_free(data);
        deinit(*source);
        deinit(*decoded);
        deinit(*reference);
        deinit(*via_detect);
    }

    // A long run of one colour uses RUN ops only.
    flat := make_image(200, 3, .RGB8);
    data := encode_qoi(flat);

    check(data.count < 14 + 8 + 20, "flat image compresses");
    decoded, decoded_ok := decode_qoi(data);

    check(decoded_ok && same_bytes(decoded.pixels, flat.pixels), "flat round trip");
    array_free(data);
    deinit(*decoded);
    deinit(*flat);

    // Refusals.
    _, refused := decode_qoi(b.bytes, 7);

    check(!refused, "max_pixels is enforced");
    _, refused = decode_qoi(slice_of(b.bytes, 13));
    check(!refused, "truncated header");

    finish();
}

slice_of :: (data: [] u8, count: s64) -> [] u8 {
    result := data;
    result.count = count;
    return result;
}
