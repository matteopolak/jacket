// One input per process: compare public entry points, configured limits and exact-cap behavior.
#import "Basic";
F :: #import "File";
I :: #import "Extensions/Image";

jpeg_same_pixels :: (a: I.Image, b: I.Image) -> bool {
    if a.width != b.width || a.height != b.height || a.format != b.format || a.pixels.count != b.pixels.count return false;
    for byte, at: a.pixels if b.pixels[at] != byte return false;
    return true;
}

jpeg_empty_image :: (image: I.Image) -> bool {
    return image.pixels.count == 0;
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 3);
    data, found := F.read_entire_file(args[1]);

    defer free(data);
    assert(found);
    print("LIMIT %\n", I.DEFAULT_MAX_PIXELS);
    image, ok, error := I.decode_jpeg(cast([] u8) data, I.DEFAULT_MAX_PIXELS);

    defer I.deinit(*image);
    detected, detected_ok := I.decode_image(cast([] u8) data, I.DEFAULT_MAX_PIXELS);

    defer I.deinit(*detected);
    print("RESULT % % % % %\n", ok, image.width, image.height, I.bytes_per_pixel(image.format), error);
    parity := ok == detected_ok;
    if ok && detected_ok parity = parity && jpeg_same_pixels(image, detected);
    if !ok && !detected_ok parity = parity && jpeg_empty_image(image) && jpeg_empty_image(detected);

    print("PARITY %\n", parity);

    if !ok {
        print("REJECT_EMPTY %\n", jpeg_empty_image(image) && jpeg_empty_image(detected));
        return;
    }

    count := cast(s64) image.width * image.height;
    valid := image.width > 0 && image.height > 0 && count <= I.DEFAULT_MAX_PIXELS && image.pixels.count == count * I.bytes_per_pixel(image.format);

    print("INVARIANT %\n", valid);
    if !valid return;

    assert(F.write_entire_file(args[2], cast(string) image.pixels));
    limited, limited_ok := I.decode_jpeg(cast([] u8) data, count - 1);

    defer I.deinit(*limited);
    detected_limited, detected_limited_ok := I.decode_image(cast([] u8) data, count - 1);

    defer I.deinit(*detected_limited);
    print("CAP_MINUS_ONE %\n", !limited_ok && !detected_limited_ok && jpeg_empty_image(limited) && jpeg_empty_image(detected_limited));
    exact, exact_ok := I.decode_jpeg(cast([] u8) data, count);

    defer I.deinit(*exact);
    detected_exact, detected_exact_ok := I.decode_image(cast([] u8) data, count);

    defer I.deinit(*detected_exact);
    print("CAP_EXACT %\n", exact_ok && detected_exact_ok && jpeg_same_pixels(image, exact) && jpeg_same_pixels(image, detected_exact));
}
