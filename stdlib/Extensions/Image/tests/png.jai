// PNG: fixtures written by generate_png_cases.py (every colour type, depth, interlace, filter,
// colour key and palette alpha form), round trips of the encoder, and chunk-level rules.

#import "Basic";
#import "Extensions/Image";
#import "Extensions/Deflate";
#load "helpers/common.jai";
#load "fixtures/png_cases.jai";

chunk :: (b: *Builder, kind: string, body: [] u8) {
    start := b.bytes.count;

    put_be32(b, body.count);
    for 0..3 put8(b, kind[it]);
    for body put8(b, it);

    crc := crc32(byte_range(b.bytes, start + 4, body.count + 4));

    put_be32(b, crc);
}

put_be32 :: (b: *Builder, v: s64) {
    for shift: 0..3 put8(b, v >> ((3 - shift) * 8));
}

byte_range :: (data: [] u8, from: s64, count: s64) -> [] u8 {
    r: [] u8;
    r.data = data.data + from;
    r.count = count;
    return r;
}

// A grayscale 8-bit PNG of `width` x `height`, every row filtered with type 0, pixel (x, y) = x + y.
simple_png :: (width: s32, height: s32, extra_before_data: string = "", extra_after_data: string = "", corrupt: s64 = -1) -> [] u8 {
    b: Builder;
    for 0..7 put8(*b, "\x89PNG\r\n\x1a\n"[it]);

    header: Builder;

    put_be32(*header, width);
    put_be32(*header, height);
    put_all(*header, 8, 0, 0, 0, 0);
    chunk(*b, "IHDR", header.bytes);

    raw: Builder;

    for y: 0..height - 1 {
        put8(*raw, 0);
        for x: 0..width - 1 put8(*raw, x + y);
    }

    packed := deflate(raw.bytes, .ZLIB, 6);

    chunk(*b, "IDAT", packed);
    chunk(*b, "IEND", .[]);
    if corrupt >= 0 b.bytes[corrupt] ^= 0x55;

    return b.bytes;
}

main :: () {
    // Fixtures.
    for PNG_CASES {
        image, ok, error := decode_png(bytes_of(it.png));
        if !ok {
            check(false, tprint("% failed: %", it.name, error));
            continue;
        }

        check(image.width == it.width && image.height == it.height && image.format == it.format, tprint("% has the wrong shape", it.name));
        check(same_bytes(image.pixels, bytes_of(it.expected)), tprint("% decodes to the wrong pixels", it.name));

        via, via_ok := decode_image(bytes_of(it.png));

        check(via_ok && same_bytes(via.pixels, image.pixels), "decode_image finds the PNG");
        deinit(*via);
        deinit(*image);
    }

    // Round trips of the encoder at several levels.
    for format: ALL_FORMATS for size : s32.[1, 2, 5, 33] for level : s32.[0, 1, 6, 9] {
        source := random_image(size, size + 3, format, cast(u64) size);
        data := encode_png(source, level);
        decoded, ok, error := decode_png(data);

        check(ok, error);

        reference := ifx format == .BGRA8 then convert(source, .RGBA8) else source;

        check(decoded.format == reference.format && same_bytes(decoded.pixels, reference.pixels), tprint("round trip % % %", format, size, level));

        array_free(data);
        deinit(*decoded);
        deinit(*source);
    }

    // A wide image with long rows exercises every filter on real data.
    wide := random_image(300, 20, .RGBA8, 4);
    data := encode_png(wide);
    decoded, ok := decode_png(data);

    check(ok && same_bytes(decoded.pixels, wide.pixels), "wide round trip");
    array_free(data);

    // Smooth content, where Sub and Up win on different rows: horizontal ramps in some rows, vertical
    // ramps in others, over widths that leave a tail after the eight-byte steps and rows long enough
    // to be sampled sparsely.
    for format: ALL_FORMATS for width : s32.[1, 3, 7, 9, 130, 1100] for level : s32.[1, 9] {
        ramp := random_image(width, 24, format, cast(u64) width);
        bpp := bytes_per_pixel(format);

        for y: 0..23 for x : 0..width - 1 for c : 0..bpp - 1 {
            value := ifx y % 2 == 0 then x * (c + 1) else y * 5 + c;
            ramp.pixels[(y * width + x) * bpp + c] = cast,trunc(u8) value;
        }

        packed := encode_png(ramp, level);
        back, back_ok := decode_png(packed);
        reference := ifx format == .BGRA8 then convert(ramp, .RGBA8) else ramp;

        check(back_ok && same_bytes(back.pixels, reference.pixels), tprint("ramp round trip % % %", format, width, level));
        array_free(packed);
        deinit(*back);
        deinit(*ramp);
    }

    // Every filter the default levels can pick has to be chosen somewhere, or its loop is untested:
    // a smooth paraboloid (Paeth), noise around a flat level (Average, which halves the noise of its
    // two neighbours), flat blocks (Up) and diagonal stripes (Sub).
    for format: s32.[0, 1, 2] {
        mixed := random_image(203, 120, ALL_FORMATS[format], 9);
        bpp := bytes_per_pixel(mixed.format);
        noise: u32 = 777;

        for y: 0..119 for x : 0..202 for c : 0..bpp - 1 {
            noise = noise * 1664525 + 1013904223;
            value := 0;

            if y < 30 {
                value = (x * x + y * y) / 64 + c;
            } else if y < 60 {
                value = 100 + cast(s64) ((noise >> 24) & 15);
            } else if y < 90 {
                value = ifx ((x / 17) + (y / 13)) % 2 == 0 then 200 else 20;
            } else {
                value = ifx ((x + y) / 4) % 2 == 0 then 200 else 20;
            }

            mixed.pixels[(y * 203 + x) * bpp + c] = cast,trunc(u8) value;
        }

        for level: s32.[1, 6] {
            packed := encode_png(mixed, level);
            back, back_ok := decode_png(packed);

            check(back_ok && same_bytes(back.pixels, mixed.pixels), tprint("mixed round trip % %", mixed.format, level));

            // The filter byte of every row, read from the inflated image data (one IDAT follows the header).
            idat_length: s64;
            for 33..36 idat_length = idat_length * 256 + packed[it];

            raw, raw_ok := inflate(byte_range(packed, 41, idat_length));
            used: [5] bool;

            check(raw_ok, "the image data inflates");

            for y: 0..119 used[raw[y * (203 * bpp + 1)]] = true;

            check(used[1] && used[2] && used[3] && used[4], tprint("every filter is chosen % %: % % % %", mixed.format, level, used[1], used[2], used[3], used[4]));
            array_free(raw);
            array_free(packed);
            deinit(*back);
        }

        deinit(*mixed);
    }

    // Rules.
    good := simple_png(4, 3);
    image, ok2, _ := decode_png(good);

    check(ok2 && image.pixels[0] == 0 && image.pixels[11] == 5, "the simple file decodes");

    for 0..good.count - 1 {
        // A change inside a chunk is caught by its checksum, or by the structure around it.
        copy := NewArray(good.count, u8);

        memcpy(copy.data, good.data, good.count);
        copy[it] ^= 0x01;
        _, changed_ok, _ := decode_png(copy);
        if it >= 8 && it < 33 check(!changed_ok, tprint("flip in the header at %", it));
    }

    // Unknown critical chunk, bad CRC, missing parts.
    insert_chunk :: (original: [] u8, before: s64, kind: string) -> [] u8 {
        b: Builder;
        for 0..before - 1 put8(*b, original[it]);

        chunk(*b, kind, .[]);
        for before..original.count - 1 put8(*b, original[it]);

        return b.bytes;
    }

    // IHDR ends at byte 33.
    with_ancillary := insert_chunk(good, 33, "tEXt");
    _, accepted, _ := decode_png(with_ancillary);

    check(accepted, "unknown ancillary chunk is ignored");
    with_critical := insert_chunk(good, 33, "XYZW");

    _, accepted, _ = decode_png(with_critical);
    check(!accepted, "unknown critical chunk is rejected");
    without_end := byte_range(good, 0, good.count - 12);

    _, accepted, _ = decode_png(without_end);
    check(!accepted, "missing IEND is rejected");
    _, accepted, _ = decode_png(byte_range(good, 0, 20));
    check(!accepted, "truncated header");

    // Sizes: zero, too big, and a size the data cannot fill.
    zero := simple_png(4, 3);
    zero[19] = 0;
    _, accepted, _ = decode_png(zero);
    check(!accepted, "zero width");
    lie := simple_png(4, 3);
    lie[23] = 4; // 4 x 4 instead of 4 x 3 (the checksum is now wrong too)
    _, accepted, _ = decode_png(lie);
    check(!accepted, "wrong height");
    _, accepted, _ = decode_png(good, 11);
    check(!accepted, "max_pixels");
    _, accepted, _ = decode_png(good, 12);
    check(accepted, "max_pixels exactly");

    finish();
}
