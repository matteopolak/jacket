// read_image_info: the header of a PNG or JPEG gives format, size and channels without decoding
// or allocating, and bad headers fail the way the decoders do.

#import "Basic";
#import "Extensions/Image";
#import "File";
#import "String";
#import "Extensions/Deflate";
#load "helpers/common.jai";
#load "fixtures/png_cases.jai";
#load "fixtures/jpeg/fixtures.jai";
#load "../../test-support/counting-allocator.jai";

PNG_HEADER_END :: 8 + 25; // signature, then IHDR: length, type, 13 data bytes, checksum

byte_range :: (data: [] u8, from: s64, count: s64) -> [] u8 {
    r: [] u8;
    r.data = data.data + from;
    r.count = count;
    return r;
}

// Probes under the tracking allocator and checks that nothing was allocated.
probe :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> Image_Info, bool, string {
    tally: Tally;
    info: Image_Info;
    ok: bool;
    error: string;

    {
        push_allocator(tally_proc, *tally);
        info, ok, error = read_image_info(data, max_pixels);
    }

    check(tally.allocations == 0, "read_image_info allocates nothing");

    return info, ok, error;
}

// Rewrites IHDR bytes of a PNG and fixes its checksum.
with_header :: (png: [] u8, at: s64, value: u8) -> [] u8 {
    copy := NewArray(png.count, u8);

    memcpy(copy.data, png.data, png.count);
    copy[16 + at] = value;

    crc := crc32(byte_range(copy, 12, 17));

    for shift: 0..3 copy[29 + shift] = cast,trunc(u8) (crc >> ((3 - shift) * 8));

    return copy;
}

main :: () {
    // Every generated PNG reports the size the decoder produces.
    for PNG_CASES {
        data := bytes_of(it.png);
        info, ok, error := probe(data);

        check(ok && info.format == .PNG && info.width == it.width && info.height == it.height, tprint("png header of %: %", it.name, error));
    }

    rgba := encode_png(make_image(5, 3, .RGBA8));
    gray := encode_png(make_image(2, 2, .GRAY8));

    defer array_free(rgba);
    defer array_free(gray);

    info, ok, error := probe(rgba);

    check(ok && info.width == 5 && info.height == 3 && info.channels == 4, "RGBA channels");
    info, ok, error = probe(gray);
    check(ok && info.channels == 1, "gray channels");

    // The header is all that is read: a PNG cut after IHDR still has one, and shorter ones do not.
    for n: 0..rgba.count - 1 {
        _, cut_ok, cut_error := probe(byte_range(rgba, 0, n));

        check(cut_ok == (n >= PNG_HEADER_END), tprint("png cut at %", n));
        check(cut_ok || cut_error.count > 0, "a failed probe says why");
    }

    // Corrupt headers.
    flipped := NewArray(rgba.count, u8);

    defer array_free(flipped);
    memcpy(flipped.data, rgba.data, rgba.count);
    flipped[20] ^= 1;
    _, ok, _ = probe(flipped);
    check(!ok, "IHDR checksum mismatch");

    for byte: PNG_HEADER_END - 17..PNG_HEADER_END - 5 {
        changed := NewArray(rgba.count, u8);

        defer array_free(changed);
        memcpy(changed.data, rgba.data, rgba.count);
        changed[byte] ^= 0x10;
        _, ok, _ = probe(changed);
        check(!ok, tprint("IHDR byte % changed without its checksum", byte));
    }

    zero_width := with_header(rgba, 3, 0);
    zero_height := with_header(rgba, 7, 0);
    odd_depth := with_header(rgba, 8, 3);
    palette := with_header(rgba, 9, 3);
    sixteen_palette := with_header(palette, 8, 16); // palette images stop at 8 bits
    interlace := with_header(rgba, 12, 2);

    defer array_free(zero_width);
    defer array_free(zero_height);
    defer array_free(odd_depth);
    defer array_free(palette);
    defer array_free(sixteen_palette);
    defer array_free(interlace);

    _, ok, _ = probe(zero_width);
    check(!ok, "zero width");
    _, ok, _ = probe(zero_height);
    check(!ok, "zero height");
    _, ok, _ = probe(odd_depth);
    check(!ok, "bit depth 3");
    _, ok, _ = probe(sixteen_palette);
    check(!ok, "16-bit palette");
    _, ok, _ = probe(interlace);
    check(!ok, "interlace method 2");

    // Chunk order: a first chunk that is not IHDR.
    renamed := NewArray(rgba.count, u8);

    defer array_free(renamed);
    memcpy(renamed.data, rgba.data, rgba.count);
    renamed[12] = #char "X";
    _, ok, _ = probe(renamed);
    check(!ok, "first chunk is not IHDR");

    // The pixel limit.
    _, ok, _ = probe(rgba, 15);
    check(ok, "exactly at the limit");
    _, ok, error = probe(rgba, 14);
    check(!ok && error.count > 0, "over the limit");

    // JPEG: size and components match the decoder for every fixture it accepts.
    for fixture: jpeg_fixtures {
        data := jpeg_bytes(fixture.jpeg);
        image, decoded, _ := decode_image(data);

        info, ok, error = probe(data);

        if decoded {
            check(ok && info.format == .JPEG && info.width == image.width && info.height == image.height, tprint("jpeg header of %: %", fixture.name, error));
            check(ok && (info.channels == 1) == (image.format == .GRAY8), tprint("jpeg channels of %", fixture.name));
        }

        deinit(*image);
    }

    for name: string.["arithmetic", "twelve_bit", "lossless"] {
        for fixture: jpeg_fixtures if fixture.name == name {
            _, ok, error = probe(jpeg_bytes(fixture.jpeg));
            check(!ok && error.count > 0, tprint("unsupported jpeg process %", name));
        }
    }

    cmyk_found := false;

    for fixture: jpeg_fixtures if fixture.name == "cmyk" {
        info, ok, error = probe(jpeg_bytes(fixture.jpeg));
        check(ok && info.channels == 4, "cmyk channels");
        cmyk_found = true;
    }

    check(cmyk_found, "cmyk fixture");

    for fixture: jpeg_fixtures if fixture.name == "odd17x9" {
        source := jpeg_bytes(fixture.jpeg);
        frame := jpeg_find_marker(source, 0xC0);
        scan := jpeg_find_marker(source, 0xDA);
        frame_end := frame + 2 + ((cast(s64) source[frame + 2]) << 8) + source[frame + 3];

        // Cut inside the frame header and before it: no size to report.
        for n: 0..frame_end - 1 {
            _, ok, _ = probe(byte_range(source, 0, n));
            check(!ok, tprint("jpeg cut at % is inside the headers", n));
        }

        // Cut anywhere after the frame header: the header is intact.
        for n: frame_end..scan {
            info, ok, _ = probe(byte_range(source, 0, n));
            check(ok && info.width == 17 && info.height == 9, tprint("jpeg cut at % keeps its frame header", n));
        }

        // Deferred height (DNL) resolves as the decoder does.
        entropy := scan + 2 + ((cast(s64) source[scan + 2]) << 8) + source[scan + 3];
        end := jpeg_entropy_end(source, entropy);
        marker := u8.[0xFF, 0xDC, 0, 4, 0, 9];
        deferred := NewArray(source.count + marker.count, u8);

        defer array_free(deferred);
        memcpy(deferred.data, source.data, end);
        memcpy(deferred.data + end, marker.data, marker.count);
        memcpy(deferred.data + end + marker.count, source.data + end, source.count - end);
        deferred[frame + 5] = 0;
        deferred[frame + 6] = 0;
        info, ok, error = probe(deferred);
        check(ok && info.width == 17 && info.height == 9, tprint("deferred height: %", error));
        _, ok, _ = probe(deferred, 17 * 9 - 1);
        check(!ok, "deferred height obeys the limit");

        no_dnl := NewArray(source.count, u8);

        defer array_free(no_dnl);
        memcpy(no_dnl.data, source.data, source.count);
        no_dnl[frame + 5] = 0;
        no_dnl[frame + 6] = 0;
        _, ok, _ = probe(no_dnl);
        check(!ok, "deferred height without DNL");

        zero_width_jpeg := NewArray(source.count, u8);

        defer array_free(zero_width_jpeg);
        memcpy(zero_width_jpeg.data, source.data, source.count);
        zero_width_jpeg[frame + 7] = 0;
        zero_width_jpeg[frame + 8] = 0;
        _, ok, _ = probe(zero_width_jpeg);
        check(!ok, "jpeg zero width");
        _, ok, _ = probe(source, 17 * 9);
        check(ok, "jpeg exactly at the limit");
        _, ok, _ = probe(source, 17 * 9 - 1);
        check(!ok, "jpeg over the limit");
    }

    // Anything else.
    _, ok, error = probe(bytes_of("not an image"));
    check(!ok && error.count > 0, "unknown data");
    _, ok, error = probe(encode_qoi(make_image(2, 2, .RGB8)));
    check(!ok && error.count > 0, "qoi is not read");
    _, ok, _ = probe(bytes_of(""));
    check(!ok, "empty input");
    _, ok, _ = probe(bytes_of("\xff\xd8\xff"));
    check(!ok, "jpeg signature alone");

    finish();
}
