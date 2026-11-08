// Deferred JPEG height must be resolved before allocating component planes.
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Image";
#load "fixtures/jpeg/fixtures.jai";

failures: int;

check :: (ok: bool, label: string) {
    if !ok {
        print("FAIL: %\n", label);
        failures += 1;
    }
}

insert :: (data: [] u8, at: int, addition: [] u8) -> [] u8 {
    result := NewArray(data.count + addition.count, u8);

    memcpy(result.data, data.data, at);
    memcpy(result.data + at, addition.data, addition.count);
    memcpy(result.data + at + addition.count, data.data + at, data.count - at);

    return result;
}

rejected :: (data: [] u8, label: string) {
    image, ok, error := decode_jpeg(data, DEFAULT_MAX_PIXELS);
    defer deinit(*image);
    check(!ok && error.count > 0 && image.pixels.count == 0, label);
}

main :: () {
    for fixture: jpeg_fixtures {
        if fixture.name != "header_ycc" && fixture.name != "odd17x9" && fixture.name != "progressive420" && fixture.name != "gray_progressive" continue;

        source := jpeg_bytes(fixture.jpeg);
        frame := jpeg_find_marker(source, 0xC0);

        if frame < 0 frame = jpeg_find_marker(source, 0xC2);

        scan := jpeg_find_marker(source, 0xDA);
        entropy := scan + 2 + ((cast(int) source[scan + 2]) << 8) + source[scan + 3];
        end := jpeg_entropy_end(source, entropy);
        height := ((cast(int) source[frame + 5]) << 8) + source[frame + 6];
        marker := u8.[0xFF, 0xDC, 0, 4, cast,trunc(u8) (height >> 8), cast,trunc(u8) height];
        deferred := insert(source, end, marker);

        defer array_free(deferred);
        deferred[frame + 5] = 0;
        deferred[frame + 6] = 0;
        original, original_ok := decode_jpeg(source, DEFAULT_MAX_PIXELS);

        defer deinit(*original);
        image, ok, error := decode_jpeg(deferred, DEFAULT_MAX_PIXELS);

        defer deinit(*image);
        check(original_ok && ok, tprint("deferred height %: %", fixture.name, error));

        if original_ok && ok {
            same := original.width == image.width && original.height == image.height && original.format == image.format && original.pixels.count == image.pixels.count;
            if same for byte, index: original.pixels if byte != image.pixels[index] same = false;

            check(same, "DNL preserves the independently checked scan pixels");
            limited, limited_ok := decode_jpeg(deferred, cast(s64) image.width * image.height - 1);

            defer deinit(*limited);
            check(!limited_ok, "deferred height obeys the pixel cap before allocation");
        }

        extended_marker := u8.[0xFF, 0xDC, 0, 6, cast,trunc(u8) (height >> 8), cast,trunc(u8) height, 0x12, 0x34];
        extended := insert(source, end, extended_marker);

        defer array_free(extended);
        extended[frame + 5] = 0;
        extended[frame + 6] = 0;
        extended_image, extended_ok := decode_jpeg(extended, DEFAULT_MAX_PIXELS);

        defer deinit(*extended_image);
        check(extended_ok, "DNL permits the segment length range in Table B.10");
        redefined := insert(source, end, marker);

        defer array_free(redefined);
        redefined[frame + 5] = 0;
        redefined[frame + 6] = cast(u8) (height + 8);
        rejected(redefined, "DNL with a nonzero frame height");
        same_height := insert(source, end, marker);

        defer array_free(same_height);
        rejected(same_height, "DNL repeating a nonzero frame height");
        duplicate := insert(deferred, end + 6, marker);

        defer array_free(duplicate);
        rejected(duplicate, "duplicate DNL");
        misplaced := insert(source, scan, marker);

        defer array_free(misplaced);
        rejected(misplaced, "DNL precedes first scan");
        no_height := NewArray(source.count, u8);

        defer array_free(no_height);
        memcpy(no_height.data, source.data, source.count);
        no_height[frame + 5] = 0;
        no_height[frame + 6] = 0;
        rejected(no_height, "missing DNL for a deferred height");
        deferred[end + 4] = 0;
        deferred[end + 5] = 0;
        rejected(deferred, "DNL has zero lines");
        deferred[end + 3] = 3;
        rejected(deferred, "DNL has a short segment");
    }

    if failures != 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
