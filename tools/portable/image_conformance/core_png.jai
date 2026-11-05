#import "Basic";
F :: #import "File";
Image :: #import "Extensions/Image";
#load "cases.jai";

failures: int;

check :: (ok: bool, name: string, reason: string) {
    if !ok {
        failures += 1;
        print("FAIL %: %\n", name, reason);
    }
}

write_bytes :: (bytes: [] u8, folder: string, name: string, suffix: string) {
    defer free(bytes.data);
    path := sprint("%/%.%", folder, name, suffix);
    defer free(path);
    check(bytes.count > 0 && F.write_entire_file(path, cast(string) bytes), name, suffix);
}

encode_variant :: (image: Image.Image, folder: string, name: string, variant: string) {
    for level: 0..9 {
        suffix := sprint("%.%.png", variant, level);
        defer free(suffix);
        write_bytes(Image.encode_png(image, cast(s32) level), folder, name, suffix);
    }

    suffix := sprint("%.bmp", variant);

    defer free(suffix);
    write_bytes(Image.encode_bmp(image), folder, name, suffix);
    qoi_suffix := sprint("%.qoi", variant);

    defer free(qoi_suffix);
    write_bytes(Image.encode_qoi(image), folder, name, qoi_suffix);
    pnm_suffix := sprint("%.pnm", variant);

    defer free(pnm_suffix);
    write_bytes(Image.encode_ppm(image), folder, name, pnm_suffix);
}

same_pixels :: (image: Image.Image, expected: string) -> bool {
    channels := Image.bytes_per_pixel(image.format);
    if image.pixels.count != image.width * image.height * channels || expected.count != image.width * image.height * 4 return false;

    for pixel: 0..cast(s64) image.width * image.height - 1 {
        gray := image.format == .GRAY8 || image.format == .GRAY_ALPHA8;

        for channel: 0..2 {
            at := pixel * channels + ifx gray then 0 else channel;
            if image.pixels[at] != expected[pixel * 4 + channel] return false;
        }

        alpha: u8 = 255;
        if image.format == .GRAY_ALPHA8 alpha = image.pixels[pixel * 2 + 1];
        if image.format == .RGBA8 alpha = image.pixels[pixel * 4 + 3];
        if alpha != expected[pixel * 4 + 3] return false;
    }

    return true;
}

run_case :: (test: Case, source: string, output: string, oracle: string) {
    defer print("DONE %\n", test.name);
    path := sprint("%/%", source, test.name);

    defer free(path);
    data, read_ok := F.read_entire_file(path);

    defer free(data);
    check(read_ok, test.name, "read input");
    if !read_ok return;

    image, ok, error := Image.decode_image(cast([] u8) data, Image.DEFAULT_MAX_PIXELS);

    defer Image.deinit(*image);
    direct, direct_ok, direct_error := Image.decode_png(cast([] u8) data, Image.DEFAULT_MAX_PIXELS);

    defer Image.deinit(*direct);
    print("RESULT % % % % % %\n", test.name, ok, direct_ok, image.width, image.height, error);
    check(ok == test.valid && direct_ok == test.valid, test.name, "full core PNG acceptance");

    if !ok || !test.valid {
        if !ok && !direct_ok check(image.pixels.count == 0 && direct.pixels.count == 0, test.name, "rejections return empty images");
        return;
    }

    check(image.width == test.width && image.height == test.height, test.name, "dimensions");
    expected_path := sprint("%/%.rgba", oracle, test.name);

    defer free(expected_path);
    expected, expected_ok := F.read_entire_file(expected_path);

    defer free(expected);
    check(expected_ok, test.name, "read independent oracle");
    if !expected_ok return;

    check(same_pixels(image, expected), test.name, "decode_image vs independent RGBA");
    check(same_pixels(direct, expected), test.name, "decode_png vs independent RGBA");
    limited, limited_ok := Image.decode_png(cast([] u8) data, cast(s64) test.width * test.height - 1);

    defer Image.deinit(*limited);
    check(!limited_ok, test.name, "pixel limit");
    detected_limited, detected_limited_ok := Image.decode_image(cast([] u8) data, cast(s64) test.width * test.height - 1);

    defer Image.deinit(*detected_limited);
    check(!detected_limited_ok && detected_limited.pixels.count == 0 && limited.pixels.count == 0, test.name, "direct and detected cap rejection is empty");
    exact, exact_ok := Image.decode_png(cast([] u8) data, cast(s64) test.width * test.height);

    defer Image.deinit(*exact);
    detected_exact, detected_exact_ok := Image.decode_image(cast([] u8) data, cast(s64) test.width * test.height);

    defer Image.deinit(*detected_exact);
    check(exact_ok && detected_exact_ok && same_pixels(exact, expected) && same_pixels(detected_exact, expected), test.name, "direct and detected exact-cap pixels");
    encode_variant(image, output, test.name, "direct");
    bgra := Image.convert(image, .BGRA8);

    defer Image.deinit(*bgra);
    encode_variant(bgra, output, test.name, "bgra");
    print("CORE % accepted\n", test.name);
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4);
    for CASES run_case(it, args[1], args[2], args[3]);

    print("Core PNG: % cases, % failures\n", CASES.count, failures);
    assert(failures == 0);
}
