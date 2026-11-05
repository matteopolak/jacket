#import "Basic";
F :: #import "File";
Image :: #import "Extensions/Image";
#load "decode_cases.jai";

failures: int;

check :: (ok: bool, name: string, reason: string) {
    if !ok {
        failures += 1;
        print("FAIL %: %\n", name, reason);
    }
}

as_rgba :: (image: Image.Image) -> string {
    result := alloc_string(cast(s64) image.width * image.height * 4);
    channels := Image.bytes_per_pixel(image.format);

    for pixel: 0..cast(s64) image.width * image.height - 1 {
        gray := image.format == .GRAY8 || image.format == .GRAY_ALPHA8;

        for channel: 0..2 {
            source_channel := ifx gray then 0 else channel;
            if image.format == .BGRA8 source_channel = 2 - channel;
            result[pixel * 4 + channel] = image.pixels[pixel * channels + source_channel];
        }

        result[pixel * 4 + 3] = 255;
        if image.format == .GRAY_ALPHA8 result[pixel * 4 + 3] = image.pixels[pixel * 2 + 1];
        if image.format == .RGBA8 || image.format == .BGRA8 result[pixel * 4 + 3] = image.pixels[pixel * 4 + 3];
    }

    return result;
}

decode_direct :: (codec: Codec, data: [] u8, cap: s64) -> Image.Image, bool {
    if codec == .BMP {
        image, ok := Image.decode_bmp(data, cap);
        return image, ok;
    }

    if codec == .PNM {
        image, ok := Image.decode_pnm(data, cap);
        return image, ok;
    }

    image, ok := Image.decode_qoi(data, cap);

    return image, ok;
}

run_case :: (test: File_Case, source: string, output: string, oracle: string) {
    defer print("DONE %\n", test.name);
    path := sprint("%/%", source, test.name);

    defer free(path);
    data, read_ok := F.read_entire_file(path);

    defer free(data);
    check(read_ok, test.name, "read input");
    if !read_ok return;

    image, ok, error := Image.decode_image(cast([] u8) data, Image.DEFAULT_MAX_PIXELS);

    defer Image.deinit(*image);
    direct, direct_ok := decode_direct(test.codec, cast([] u8) data, Image.DEFAULT_MAX_PIXELS);

    defer Image.deinit(*direct);
    check(direct_ok == ok, test.name, "direct and detected entry points");
    status := ifx ok then 1 else 0;

    print("RESULT % % % % %\n", test.name, status, image.width, image.height, error);
    if test.required >= 0 check(ok == (test.required == 1), test.name, "required acceptance policy");

    if !ok {
        check(image.pixels.count == 0 && direct.pixels.count == 0, test.name, "rejections return empty images");
        return;
    }

    channels := Image.bytes_per_pixel(image.format);
    valid := image.width > 0 && image.height > 0 && channels > 0 && cast(s64) image.width * image.height <= Image.DEFAULT_MAX_PIXELS && image.pixels.count == cast(s64) image.width * image.height * channels;

    check(valid, test.name, "decoded image invariant and pixel cap");
    if !valid return;

    raw := as_rgba(image);

    defer free(raw);

    if direct_ok {
        direct_valid := direct.width == image.width && direct.height == image.height && direct.pixels.count == cast(s64) direct.width * direct.height * Image.bytes_per_pixel(direct.format);

        check(direct_valid, test.name, "direct decoded invariant");

        if direct_valid {
            direct_raw := as_rgba(direct);

            defer free(direct_raw);
            same := direct_raw.count == raw.count;
            if same for byte, at: raw if direct_raw[at] != byte same = false;

            check(same, test.name, "direct and detected pixels");
        }
    }

    destination := sprint("%/%.rgba", output, test.name);

    defer free(destination);
    check(F.write_entire_file(destination, raw), test.name, "write raw independent comparison output");
    limited, limited_ok := Image.decode_image(cast([] u8) data, cast(s64) image.width * image.height - 1);

    defer Image.deinit(*limited);
    check(!limited_ok, test.name, "pixel limit rejection");
    direct_limited, direct_limited_ok := decode_direct(test.codec, cast([] u8) data, cast(s64) image.width * image.height - 1);

    defer Image.deinit(*direct_limited);
    check(!direct_limited_ok, test.name, "direct pixel limit rejection");
    exact_cap := cast(s64) image.width * image.height;
    exact, exact_ok := Image.decode_image(cast([] u8) data, exact_cap);

    defer Image.deinit(*exact);
    direct_exact, direct_exact_ok := decode_direct(test.codec, cast([] u8) data, exact_cap);

    defer Image.deinit(*direct_exact);
    check(exact_ok && direct_exact_ok, test.name, "direct and detected exact-cap acceptance");

    if exact_ok && direct_exact_ok {
        exact_raw := as_rgba(exact);

        defer free(exact_raw);
        direct_raw := as_rgba(direct_exact);

        defer free(direct_raw);
        same := exact_raw.count == raw.count && direct_raw.count == raw.count;
        if same for byte, at: raw if exact_raw[at] != byte || direct_raw[at] != byte same = false;

        check(same, test.name, "direct and detected exact-cap pixels");
    }

    if test.required != 1 return;

    check(image.width == test.width && image.height == test.height, test.name, "reference dimensions");
    expected_path := sprint("%/%.rgba", oracle, test.name);

    defer free(expected_path);
    expected, expected_ok := F.read_entire_file(expected_path);

    defer free(expected);
    check(expected_ok && expected.count == raw.count, test.name, "independent oracle length");
    if !expected_ok || expected.count != raw.count return;

    largest: s32;

    for actual, at: raw {
        delta := cast(s32) actual - expected[at];
        if delta < 0 delta = -delta;
        if delta > largest largest = delta;
    }

    if largest > test.delta print("PIXEL % maximum_delta % permitted %\n", test.name, largest, test.delta);

    check(largest <= test.delta, test.name, "independent pixels");

}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4);
    for CASES run_case(it, args[1], args[2], args[3]);

    print("Decoder corpus: % cases, % failures\n", CASES.count, failures);
    assert(failures == 0);
}
