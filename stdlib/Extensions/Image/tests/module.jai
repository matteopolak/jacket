// The module's entry points: format detection, decode_image, decode_image_as, make_image and the
// format helpers.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

main :: () {
    // bytes_per_pixel and has_alpha.
    check(bytes_per_pixel(.GRAY8) == 1 && bytes_per_pixel(.GRAY_ALPHA8) == 2 && bytes_per_pixel(.RGB8) == 3, "bytes_per_pixel");
    check(bytes_per_pixel(.RGBA8) == 4 && bytes_per_pixel(.BGRA8) == 4, "bytes_per_pixel with alpha");
    check(bytes_per_pixel(cast,no_check(Pixel_Format) 200) == 0, "an unknown format has no size");
    check(!has_alpha(.GRAY8) && !has_alpha(.RGB8) && has_alpha(.GRAY_ALPHA8) && has_alpha(.RGBA8) && has_alpha(.BGRA8), "has_alpha");

    // make_image.
    image := make_image(3, 2, .RGBA8);

    check(image.width == 3 && image.height == 2 && image.pixels.count == 24, "make_image");
    check(same_bytes(image.pixels, NewArray(24, u8)), "make_image is zeroed");
    deinit(*image);
    check(image.pixels.count == 0 && image.width == 0, "deinit resets");
    check(make_image(0, 4, .RGB8).pixels.count == 0, "zero width");
    check(make_image(4, -1, .RGB8).pixels.count == 0, "negative height");
    check(make_image(4, 4, cast,no_check(Pixel_Format) 99).pixels.count == 0, "unknown format");

    // Detection by signature, whatever the rest holds.
    check(detect_image_format(bytes_of("\x89PNG\r\n\x1a\nrest")) == .PNG, "png");
    check(detect_image_format(bytes_of("\xff\xd8\xff\xe0")) == .JPEG, "jpeg");
    check(detect_image_format(bytes_of("qoif....")) == .QOI, "qoi");
    check(detect_image_format(bytes_of("BM......")) == .BMP, "bmp");

    for kind: u8.["2"[0], "3"[0], "5"[0], "6"[0]] {
        text := tprint("P%\n", string.{ 1, *kind });
        check(detect_image_format(bytes_of(text)) == .PNM, "pnm");
    }

    check(detect_image_format(bytes_of("P4\n")) == .UNKNOWN && detect_image_format(bytes_of("P1\n")) == .UNKNOWN, "bitmap PBM is not supported");
    check(detect_image_format(bytes_of("")) == .UNKNOWN && detect_image_format(bytes_of("P")) == .UNKNOWN, "short input");
    check(detect_image_format(bytes_of("GIF89a")) == .UNKNOWN, "gif");

    // decode_image gives the stored format; decode_image_as converts.
    source := random_image(6, 4, .RGB8, 2);
    png := encode_png(source);
    qoi := encode_qoi(source);
    bmp := encode_bmp(source);
    ppm := encode_ppm(source);

    files: [..][] u8;

    array_add(*files, png);
    array_add(*files, qoi);
    array_add(*files, bmp);
    array_add(*files, ppm);

    for file: files {
        decoded, ok := decode_image(file);

        check(ok && decoded.format == .RGB8 && same_bytes(decoded.pixels, source.pixels), tprint("decode_image of file %", it_index));
        deinit(*decoded);

        for format: ALL_FORMATS {
            converted, converted_ok := decode_image_as(file, format);
            reference := convert(source, format);

            check(converted_ok && converted.format == format && same_bytes(converted.pixels, reference.pixels), tprint("decode_image_as % of file %", format, it_index));
            deinit(*converted);
            deinit(*reference);
        }

        _, bad_ok, _ := decode_image_as(file, cast,no_check(Pixel_Format) 77);

        check(!bad_ok, "decode_image_as with an unknown format");
    }

    // Failures say why, and return an empty image.
    nothing, nothing_ok, nothing_error := decode_image(bytes_of("not an image at all"));

    check(!nothing_ok && nothing.pixels.count == 0 && nothing_error.count > 0, "unknown data");
    _, jpeg_ok, jpeg_error := decode_image(bytes_of("\xff\xd8\xff"));

    check(!jpeg_ok && jpeg_error.count > 0, "a truncated JPEG fails");
    _, limited, _ := decode_image(png, 5);

    check(!limited, "max_pixels reaches the decoders");

    finish();
}
